use crate::errors::RustFlowError;
use crate::state::AppState;
use axum::extract::{OriginalUri, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use rustflow_common::AuthenticatedClient;
use std::time::Instant;

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
            if let Some(client_name) = state.api_keys.get(key) {
                request.extensions_mut().insert(AuthenticatedClient {
                    name: client_name.clone(),
                });
                next.run(request).await
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
    println!("{} {}", request.method(), uri);
    next.run(request).await
}


pub async fn log_elapsed_time(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let response = next.run(request).await;
    println!("Took {} mu", Instant::now().duration_since(start).as_micros() as f64);
    response
}


