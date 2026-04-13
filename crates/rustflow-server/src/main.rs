mod errors;
mod extractors;
mod routes;
mod state;

use crate::routes::middleware::{rate_limited, request_counter};
use crate::state::AppState;
use axum::http::{HeaderName, HeaderValue, Method, Request};
use axum::{middleware, Router};
use rustflow_common::{AuthenticatedClient, APP_NAME, APP_VERSION};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Notify;
use tower::ServiceBuilder;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::{CompressionLayer, DefaultPredicate, Predicate};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultOnResponse, MakeSpan, TraceLayer};
use tracing::field::Empty;
use tracing::{Level, Span};

/// Custom span factory for TraceLayer
///
/// For every incoming HTTP request, create a span with:
/// - `http.method` and `http.uri` (always present)
/// - `request_id` - a unique UUID v4 for correlating all events in this request
/// - `client.name` - the authenticated client's name ( if present in extension)
/// - `entity.id` - the entity ID from the URI path ( if present)

#[derive(Debug, Clone)]
struct RustFlowMakeSpan;

impl<B> MakeSpan<B> for RustFlowMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        // Generate a unique ID for this request - correlates all log lines
        let request_id = uuid::Uuid::new_v4().to_string();

        // Create the span with all fields declared upfront.
        // Optional fields use Empty - they will be filled in below if available.
        let span = tracing::debug_span!(
            "request",
            http.method = %request.method(),
            http.uri = %request.uri(),
            request_id = %request_id,
            client.name = Empty,
            entity.id = Empty,
        );

        if let Some(client) = request.extensions().get::<AuthenticatedClient>() {
            span.record("client.name", &client.name.as_str());
        }

        if let Some(id) = extract_entity_id(request.uri().path()) {
            span.record("entity.id", id.as_str());
        }
        span
    }
}

fn extract_entity_id(path: &str) -> Option<String> {
    path.split('/')
        .find(|segment| !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit()))
        .map(|id| id.to_string())
}

fn build_app(state: &AppState) -> Router {
    // CORS configuration
    let cors: CorsLayer = CorsLayer::new()
        // Each parse returns Result — filter_map discards any that fail to parse
        .allow_origin(
            state
                .config
                .cors_origins
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-api-key"),
        ])
        //       .allow_credentials(true) panics
        .expose_headers([HeaderName::from_static("x-authenticated-as")])
        .max_age(Duration::from_secs(3600));

    // ------------- Static files ------------
    // No middleware - served as fast as possible
    let static_router: Router<AppState> = Router::new()
        .route_service(
            "/",
            ServeFile::new("crates/rustflow-server/static/index.html"),
        )
        // Serve static files from /static/* — automatically serves index.html for directory requests
        .nest_service(
            "/static",
            ServeDir::new("crates/rustflow-server/static").append_index_html_on_directories(true),
        )
        .route_service(
            "/favicon.ico",
            ServeFile::new("crates/rustflow-server/static/favicon.ico"),
        );
    // ------------ Infrastructure --------------
    // Logging only - no rate limiting
    let infra_router: Router<AppState> = routes::health::router();

    let api_router: Router<AppState> = Router::new()
        .nest("/tasks", routes::tasks::router(state.clone()))
        .nest("/projects", routes::projects::router(state.clone()))
        .nest("/users", routes::users::router(state.clone()))
        .nest(
            "/enrichment",
            rate_limited(routes::enrichment::router(), 10, Duration::from_secs(10)),
        )
        .layer(
            ServiceBuilder::new()
                // outermost -> innermost
                .layer(
                    CompressionLayer::new()
                        .compress_when(DefaultPredicate::new().and(SizeAbove::new(512))),
                )
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    request_counter,
                )),
        );

    // Build the application router
    let app: Router = Router::new()
        .merge(static_router)
        .merge(infra_router)
        // Domain routes — nested under /api/*
        .nest("/api", api_router)
        // Provide state to ALL routes (merged and nested)
        .with_state(state.clone())
        .layer(
            TraceLayer::new_for_http().make_span_with(RustFlowMakeSpan).on_response(
                DefaultOnResponse::new()
                    .level(Level::DEBUG)
                    .latency_unit(tower_http::LatencyUnit::Micros),
            ),
        )
        // CORS is the only global layer — needed for all browser requests
        .layer(cors);

    app
}

#[tokio::main]
async fn main() {
    // Initialize tracing first - before any other code runs
    // All tracing macros (info!, debug!, warn!) are no-ops until this is called.
    rustflow_common::tracing::init_tracing();
    let state: AppState = AppState::new();

    let app_router: Router = build_app(&state);
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    // Shared doorbell: one handle goes into the server, one stays in main
    let shutdown_notify: Arc<Notify> = Arc::new(Notify::new());
    let server_shutdown: Arc<Notify> = shutdown_notify.clone(); // Arc clone — cheap pointer copy

    // Server will keep accepting connections until `server_shutdown` is notified
    let server = axum::serve(listener, app_router)
        .with_graceful_shutdown(async move {
            server_shutdown.notified().await; // parks here until the doorbell rings
        })
        .into_future(); // convert to a standard Future so select!/timeout can use it

    // Pin to the stack — required to poll the same future multiple times in select!
    tokio::pin!(server);

    // Race: normal server lifetime vs incoming shutdown signal
    tokio::select! {
        // Server stopped on its own (error or all listeners closed)
        result = &mut server => {
            result.expect("Server error");
            tracing::info!("{} stopped", APP_NAME);
        }
        // Ctrl+C or SIGTERM received
        _ = shutdown_signal() => {
            tracing::info!("Draining... (10s timeout)");
            state.shutdown_requested.store(true, Ordering::Relaxed);
            // Ring the doorbell → Axum stops accepting, starts draining in-flight requests
            shutdown_notify.notify_waiters();

            // Wait up to 10s for in-flight requests to finish
            match tokio::time::timeout(Duration::from_secs(10), &mut server).await {
                Ok(result) => {
                    result.expect("Server error");
                    tracing::info!("{} has shut down gracefully", APP_NAME);
                }
                Err(_) => {
                    // Requests still running after 10s — give up
                    tracing::warn!("Shutdown timeout exceeded, forcing exit");
                }
            }
        }
    }
}

/// Wait for either SIGINT (Ctrl+C) or SIGTERM (docker stop / kill -15)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // SIGTERM only exists on Unix; on other platforms use a future that never resolves
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // never completes — ctrl_c is the only option

    // Whichever signal arrives first wins
    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), starting graceful shutdown...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown...");
        }
    }
}
