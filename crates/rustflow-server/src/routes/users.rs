use crate::errors::RustFlowError;
use crate::routes::middleware::{rate_limited, require_api_key};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use rustflow_common::{CreateUser, User};
use std::time::Duration;

#[tracing::instrument(name = "user.list", skip_all)]
async fn list(State(state): State<AppState>) -> Json<Vec<User>> {
    let store = state.users.0.read().await;
    Json(store.users.clone())
}

#[tracing::instrument(name = "user.create", skip_all, fields(user.id))]
async fn create(
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

#[tracing::instrument(name = "user.delete", skip_all, fields(user.id = %id))]
async fn delete(
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

#[tracing::instrument(name = "user.get", skip_all, fields(user.id = %id))]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<(StatusCode, Json<User>), RustFlowError> {
    let store = state.users.0.read().await;
    let found_user = store.users.iter().find(|user| user.id == id);

    if let Some(user) = found_user {
        Ok((StatusCode::OK, Json(user.clone())))
    } else {
        Err(RustFlowError::NotFound(format!(
            "User with id {} not found",
            id
        )))
    }
}

pub fn router(state: AppState) -> Router<AppState> {
    let public = rate_limited(
        Router::new()
            .route("/", get(list))
            .route("/{id}", get(get_one)),
        100,
        Duration::from_secs(10),
    );

    let protected = rate_limited(
        Router::new()
            .route("/", post(create))
            .route("/{id}", axum::routing::delete(delete))
            .layer(middleware::from_fn_with_state(state, require_api_key)),
        10,
        Duration::from_secs(10),
    );

    public.merge(protected)
}
