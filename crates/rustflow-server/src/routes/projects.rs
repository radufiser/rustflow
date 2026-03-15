use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use rustflow_common::{CreateProject, Project};

/// GET /projects
async fn list(State(state): State<AppState>) -> Json<Vec<Project>> {
    let store = state.projects.0.read().await;
    Json(store.projects.clone())
}

/// GET /projects/:id
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Project>, StatusCode> {
    let store = state.projects.0.read().await;
    store
        .projects
        .iter()
        .find(|project| project.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// DELETE /projects/:id
async fn delete(State(state): State<AppState>, Path(id): Path<u64>) -> StatusCode {
    let mut store = state.projects.0.write().await;
    let len_before = store.projects.len();
    store.projects.retain(|project| project.id != id);
    if store.projects.len() < len_before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::OK
    }
}

/// POST /projects
async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateProject>,
) -> (StatusCode, Json<Project>) {
    let mut store = state.projects.0.write().await;
    let id = store.next_id;
    let project = Project {
        id,
        name: payload.name,
        description: payload.description,
    };
    store.next_id += 1;
    store.projects.push(project.clone());
    (StatusCode::CREATED, Json(project))
}


pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).delete(delete))
}
