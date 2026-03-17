// ── The unified error type ──────────────────────────────────

use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;
use thiserror::Error;

/// A single error type for all RustFlow handler errors.
///
/// Every variant maps to a specific HTTP status code and produces
/// a consistent JSON response body:
///
/// ```json
/// {
///   "error": "Short description",
///   "details": "More context about what went wrong"
/// }
/// ```
#[derive(Debug, Error)]
pub enum RustFlowError {
    #[error("Not found `{0}`")]
    NotFound(String),
    #[error("Bad request `{0}`")]
    BadRequest(String),
    #[error("Validation Failed `{0}`")]
    UnprocessableEntity(String),
    #[error("External Service Error `{0}`")]
    ExternalService(String),
    #[error("Internal Server Error `{0}`")]
    Internal(String),
    #[error("Conflict error `{0}`")]
    Conflict(String),
}

impl IntoResponse for RustFlowError {
    fn into_response(self) -> Response {
        let (status, error, details) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", msg),

            Self::UnprocessableEntity(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Validation Failed", msg)
            }

            Self::ExternalService(msg) => (StatusCode::BAD_GATEWAY, "External Service Error", msg),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                msg,
            ),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "Conflict error", msg),
        };

        let body = serde_json::json!({
            "error": error,
            "details": details
        });

        (status, Json(body)).into_response()
    }
}


