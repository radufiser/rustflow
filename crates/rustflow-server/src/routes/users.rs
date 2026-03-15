use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use rustflow_common::{CreateUser, User};

async fn list(State(state): State<AppState>) -> Json<Vec<User>> {
    let store = state.users.0.read().await;
    Json(store.users.clone())
}

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

pub fn router() -> Router<AppState> {
    Router::new().route("/users", get(list).post(create))
}
