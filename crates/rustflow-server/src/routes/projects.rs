use crate::errors::RustFlowError;
use crate::routes::middleware::{rate_limited, require_api_key};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use rustflow_common::{AuthenticatedClient, CreateProject, Project};
use std::time::Duration;

/// GET /projects
#[tracing::instrument(name = "project.list", skip_all)]
async fn list(State(state): State<AppState>) -> Json<Vec<Project>> {
    let store = state.projects.0.read().await;
    Json(store.projects.clone())
}

/// GET /projects/:id
#[tracing::instrument(name = "project.get", skip_all, fields(project.id = %id))]
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
#[tracing::instrument(name = "project.delete", skip_all, fields(project.id = %id))]
async fn delete(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, RustFlowError> {
    let mut store = state.projects.0.write().await;
    let len_before = store.projects.len();
    store.projects.retain(|project| project.id != id);
    if store.projects.len() < len_before {
        tracing::info!("[audit] Project {} deleted by client '{}'", id, client.name);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(RustFlowError::NotFound(format!(
            "Project with id {id} does not exist"
        )))
    }
}

/// POST /projects
#[tracing::instrument(name = "project.create", skip_all, fields(project.id))]
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
    tracing::info!(
        "[audit] Project {} created by client '{}'",
        project.id, client.name
    );
    tracing::span::Span::current().record("project.id", id);
    (StatusCode::CREATED, Json(project))
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
