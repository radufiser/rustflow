mod errors;
mod extractors;
mod routes;
mod state;

use crate::routes::middleware::{log_elapsed_time, log_request};
use crate::state::AppState;
use axum::error_handling::HandleErrorLayer;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::{middleware, Router};
use rustflow_common::{APP_NAME, APP_VERSION};
use std::time::Duration;
use tokio::net::TcpListener;
use tower::{
    buffer::BufferLayer, limit::RateLimitLayer, load_shed::LoadShedLayer, BoxError, ServiceBuilder,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Wraps a router with the full rate-limiting layer stack:
/// HandleError → Buffer → LoadShed → RateLimit → router
fn rate_limited<S: Clone + Send + Sync + 'static>(
    router: Router<S>,
    num: u64,
    per: Duration,
) -> Router<S> {
    let retry_after = per.as_secs().to_string();
    router.layer(
        ServiceBuilder::new()
            .layer(middleware::from_fn(move |request: Request, next: Next| {
                let retry_after = retry_after.clone();
                async move {
                    let mut response = next.run(request).await;
                    if response.status() == StatusCode::SERVICE_UNAVAILABLE {
                        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                        if let Ok(val) = HeaderValue::from_str(&retry_after) {
                            response.headers_mut().insert("Retry-After", val);
                        }
                    }
                    response
                }
            }))
            .layer(HandleErrorLayer::new(|err: BoxError| async move {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Service overloaded: {}", err),
                )
            }))
            .layer(BufferLayer::new(100))
            .layer(LoadShedLayer::new())
            .layer(RateLimitLayer::new(num, per)),
    )
}

#[tokio::main]
async fn main() {
    let state: AppState = AppState::new();

    // CORS configuration
    let cors = CorsLayer::new()
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
    let static_router = Router::new()
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
    let infra_router = routes::health::router().layer(middleware::from_fn(log_request));

    let api_router = Router::new()
        .nest(
            "/tasks",
            rate_limited(
                routes::tasks::router(state.clone()),
                50,
                Duration::from_secs(10),
            ),
        )
        .nest(
            "/projects",
            rate_limited(
                routes::projects::router(state.clone()),
                30,
                Duration::from_secs(10),
            ),
        )
        .nest(
            "/users",
            rate_limited(
                routes::users::router(state.clone()),
                20,
                Duration::from_secs(10),
            ),
        )
        .nest(
            "/enrichment",
            rate_limited(routes::enrichment::router(), 10, Duration::from_secs(10)),
        )
        .layer(
            ServiceBuilder::new()
                 // outermost -> innermost
                .layer(middleware::from_fn(log_request))
                .layer(middleware::from_fn(log_elapsed_time))
        );

    // Build the application router
    let app = Router::new()
        .merge(static_router)
        .merge(infra_router)
        // Domain routes — nested under /api/*
        .nest("/api", api_router)
        // Provide state to ALL routes (merged and nested)
        .with_state(state)
        // CORS is the only global layer — needed for all browser requests
        .layer(cors);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await.expect("Server error");
}
