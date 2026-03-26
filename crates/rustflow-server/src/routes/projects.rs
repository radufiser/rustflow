use crate::errors::RustFlowError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};
use rustflow_common::{AuthenticatedClient, CreateProject, Project};
use crate::routes::middleware::{log_elapsed_time, log_request, require_api_key};

/// GET /projects
async fn list(State(state): State<AppState>) -> Json<Vec<Project>> {
    let store = state.projects.0.read().await;
    Json(store.projects.clone())
}

/// GET /projects/:id
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Project>, RustFlowError> {
    let store = state.projects.0.read().await;
    store
        .projects
        .iter()
        .find(|project| project.id == id)
        .cloned()
        .map(Json)
        .ok_or(RustFlowError::NotFound(format!(
            "Project with id {} not found",
            id
        )))
}

/// DELETE /projects/:id
async fn delete(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, RustFlowError> {
    let mut store = state.projects.0.write().await;
    let len_before = store.projects.len();
    store.projects.retain(|project| project.id != id);
    if store.projects.len() < len_before {
        println!("[audit] Project {} deleted by client '{}'", id, client.name);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(RustFlowError::NotFound(format!(
            "Project with id {id} does not exist"
        )))
    }
}

/// POST /projects
async fn create(
    Extension(client): Extension<AuthenticatedClient>,
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
    println!("[audit] Project {} created by client '{}'", project.id, client.name);
    (StatusCode::CREATED, Json(project))
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).delete(delete))
        .layer(middleware::from_fn_with_state(state, require_api_key))
        .layer(middleware::from_fn(log_request))
        .layer(middleware::from_fn(log_elapsed_time))
}
