use std::time::Instant;
use crate::errors::RustFlowError;
use axum::extract::{OriginalUri, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use crate::state::AppState;

pub async fn require_api_key(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let api_key = request
        .headers()
        .get("X-API-KEY")
        .and_then(|value| value.to_str().ok());

    match api_key {
        None => RustFlowError::Unauthorized("Missing x-api-key header".into()).into_response(),
        Some(api_key) if api_key == state.config.api_key => next.run(request).await,
        Some(_) => RustFlowError::Forbidden("Invalid x-api-key header".into()).into_response(),
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


