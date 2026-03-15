use axum::{extract::State, routing::get, Json, Router};
use rustflow_common::AppConfig;
use crate::state::AppState;

/// Health check endpoint - returns structured JSON about the service
async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": state.config.app_name,
        "version": state.config.version,
        "status": "ok"
    }))
}

/// GET /config
///
/// Returns the read-only application configuration.
/// No lock needed — `AppConfig` is immutable after startup.
async fn config(State(state): State<AppState>) -> Json<AppConfig> {
    Json(state.config.clone())
}

/// Build the health/meta router.
///
/// Uses `.merge()` in main (no prefix) so these become:
///   GET /health
///   GET /config
pub fn router() -> Router<AppState> {
    Router::new()
    .route("/health", get(health))
    .route("/config", get(config))
}