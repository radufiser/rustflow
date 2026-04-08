use std::sync::atomic::Ordering;
use crate::errors::RustFlowError;
use crate::state::AppState;
use axum::extract::{OriginalUri, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use rustflow_common::AuthenticatedClient;
use std::time::{Duration, Instant};
use axum::http::{HeaderValue, StatusCode};
use axum::{middleware, Router};
use axum::error_handling::HandleErrorLayer;
use tower::{BoxError, ServiceBuilder};
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::load_shed::LoadShedLayer;

/// Middleware that validates the `x-api-key` header and injects the
/// authenticated client identity into request extensions.
///
/// Applied to a router with:
/// ```ignore
/// Router::new()
///     .route("/", post(create))
///     .layer(middleware::from_fn_with_state(state.clone(), require_api_key))
/// ```
///
/// After this middleware runs, handlers can extract the identity:
/// ```ignore
/// async fn create(
///     Extension(client): Extension<AuthenticatedClient>,
///     ...
/// ) {
///     println!("Created by: {}", client.name);
/// }
/// ```
pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next) -> Response {
    let api_key = request
        .headers()
        .get("X-API-KEY")
        .and_then(|value| value.to_str().ok());

    match api_key {
        None => RustFlowError::Unauthorized("Missing x-api-key header".into()).into_response(),
        Some(key) => {
            if let Some((client_name, role)) = state.api_keys.get(key) {
                request.extensions_mut().insert(AuthenticatedClient {
                    name: client_name.clone(),
                    role: role.clone(),
                });
                let mut response = next.run(request).await;
                if let Ok(value) = HeaderValue::from_str(&client_name) {
                    response.headers_mut().append("x-authenticated-as", value);
                }
                response
            } else {
                RustFlowError::Forbidden("Invalid x-api-key header".into()).into_response()
            }
        }
    }
}


pub async fn log_request(request: Request, next: Next) -> Response {
    // Use OriginalUri from extensions to get the full path before nest() strips the prefix
    let uri = request
        .extensions()
        .get::<OriginalUri>()
        .map(|original| original.0.to_string())
        .unwrap_or_else(|| request.uri().to_string());
    tracing::info!("{} {}", request.method(), uri);
    next.run(request).await
}


pub async fn log_elapsed_time(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let response = next.run(request).await;
    tracing::debug!("Took {}μs", start.elapsed().as_micros());
    response
}

/// Wraps a router with the full rate-limiting layer stack:
/// HandleError → Buffer → LoadShed → RateLimit → router
pub fn rate_limited<S: Clone + Send + Sync + 'static>(
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

pub async fn request_counter(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let prev = state.request_counter.fetch_add(1, Ordering::SeqCst);
    let new = prev + 1;
    if new % 100 == 0 {
        tracing::debug!("Request counter: {}", new);
    }
    next.run(request).await
}