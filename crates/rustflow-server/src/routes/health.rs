use crate::state::AppState;
use axum::http::StatusCode;
use axum::{extract::State, routing::get, Extension, Json, Router};
use rustflow_common::{AppConfig, AuthenticatedClient};
use std::sync::atomic::Ordering;

/// Health check endpoint - returns structured JSON about the service.
///
/// Demonstrates **optional extension extraction**: this route has NO auth
/// middleware, so `AuthenticatedClient` is never injected.  Using
/// `Option<Extension<T>>` lets the handler work either way — `None` when
/// unauthenticated, `Some(client)` if the middleware happened to run.
///
/// If we used `Extension<AuthenticatedClient>` (non-optional) here, Axum
/// would return `500 Internal Server Error` because the extension is missing.
async fn health(
    auth: Option<Extension<AuthenticatedClient>>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let caller = match auth {
        Some(Extension(client)) => client.name,
        None => "anonymous".to_string(),
    };

    Json(serde_json::json!({
        "service": state.config.app_name,
        "version": state.config.version,
        "status": "ok",
        "caller": caller
    }))
}

/// GET /config
///
/// Returns the read-only application configuration.
/// No lock needed — `AppConfig` is immutable after startup.
async fn config(State(state): State<AppState>) -> Json<AppConfig> {
    Json(state.config.clone())
}

async fn ready(State(state): State<AppState>) -> StatusCode {
    if state.shutdown_requested.load(Ordering::Relaxed) {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    }
}

/// Build the health/meta router.
///
/// Uses `.merge()` in main (no prefix) so these become:
///   GET /health
///   GET /config
///   GET /ready
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/config", get(config))
        .route("/ready", get(ready))
}
