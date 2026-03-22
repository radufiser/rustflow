use crate::errors::RustFlowError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use rustflow_common::{CreateUser, User};
use crate::extractors::RequireApiKey;

async fn list(State(state): State<AppState>) -> Json<Vec<User>> {
    let store = state.users.0.read().await;
    Json(store.users.clone())
}

async fn create(
    _auth: RequireApiKey,
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let mut store = state.users.0.write().await;

    let user = User {
        id: store.next_id,
        email: payload.email,
        first_name: payload.first_name,
        last_name: payload.last_name,
    };

    store.next_id += 1;
    store.users.push(user.clone());
    (StatusCode::CREATED, Json(user))
}

async fn delete(
    _auth: RequireApiKey,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, RustFlowError> {
    let mut store = state.users.0.write().await;
    let len_before = store.users.len();
    store.users.retain(|user| user.id != id);

    if store.users.len() < len_before {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(RustFlowError::NotFound(format!(
            "User with id {} not found",
            id
        )))
    }
}

async fn get_one(State(state): State<AppState>, Path(id): Path<u64>) -> Result<(StatusCode, Json<User>), RustFlowError> {
    let store = state.users.0.read().await;
    let found_user = store.users.iter().find(|user| user.id == id);

    if let Some(user) = found_user {
        Ok((StatusCode::OK, Json(user.clone())))
    } else {
        Err(RustFlowError::NotFound(format!("User with id {} not found", id)))
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list).post(create))
        .route("/users/{id}", get(get_one).delete(delete))
}
