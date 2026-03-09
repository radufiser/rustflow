use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// This is a tuple struct -  a struct over an unnamed field.
/// Think of it as a wrapper around any type `T`.
///
pub struct ValidatedJson<T>(pub T);

/// What can go wrong?
pub enum ValidatedJsonError {
    // the JSON itself is broken
    JsonError(JsonRejection),
    // the JSON is valid but breaks out rules
    ValidationError(validator::ValidationErrors),
}

impl IntoResponse for ValidatedJsonError {
    fn into_response(self) -> Response {
        match self {
            // Bad JSON? Let Axum's built-in handler deal with it (returns 400)
            Self::JsonError(rejection) => rejection.into_response(),
            Self::ValidationError(errors) => {
                // ... builds a JSON body like:
                // { "error": "Validation failed", "details": { "title": ["Title must be..."] } }
                // ... and returns it with status 422
                let body = serde_json::json!({
                    "error": "Validation failed",
                    "details": errors.field_errors().iter().map(|(field, errs)| {
                        (field.to_string(), errs.iter().filter_map(|e| {
                            e.message.as_ref().map(|m| m.to_string())
                        }).collect::<Vec<_>>())
                    }).collect::<std::collections::HashMap<_, _>>()
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }
        }
    }
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync, // S (server state) must be thread safe
{
    type Rejection = ValidatedJsonError; // What error type we return on failure

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Step 1: Try to parse the JSON body into type T
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidatedJsonError::JsonError)?;
        // Step 2: Run the validation rules on the parsed values
        value
            .validate()
            .map_err(ValidatedJsonError::ValidationError)?;

        // Step 3: both passed.
        Ok(ValidatedJson(value))
    }
}
