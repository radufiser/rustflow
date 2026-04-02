use crate::errors::RustFlowError;
use crate::extractors::{ValidatedJson, ValidatedQuery};
use crate::routes::middleware::{rate_limited, require_api_key};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post, put};
use axum::{middleware, Extension, Json, Router};
use rustflow_common::{AuthenticatedClient, CreateTask, Task, TaskFilter, TaskStatus};
use std::time::Duration;

// ── Read handlers (public) ──────────────────────────────────

/// GET /tasks?status=pending&priority=high
///
/// Uses `Option<Extension<AuthenticatedClient>>` so it works with or without
/// the auth middleware. When a key is provided, the client name is logged.
async fn list(
    auth: Option<Extension<AuthenticatedClient>>,
    State(state): State<AppState>,
    ValidatedQuery(filter): ValidatedQuery<TaskFilter>,
) -> Json<Vec<Task>> {
    if let Some(Extension(client)) = auth {
        println!("Listing tasks for authenticated client {}", client.name);
    }
    let state = state.tasks.0.read().await;

    let filtered: Vec<Task> = state
        .tasks
        .iter()
        .filter(|task| filter.status.as_ref().map_or(true, |s| &task.status == s))
        .filter(|task| {
            filter
                .priority
                .as_ref()
                .map_or(true, |p| &task.priority == p)
        })
        .filter(|task| {
            filter.search.as_ref().map_or(true, |s| {
                task.title.contains(s) || task.description.as_ref().map_or(false, |d| d.contains(s))
            })
        })
        .cloned()
        .collect();

    Json(filtered)
}

/// GET /tasks/:id
///
/// The Path extractor captures the `id` from the URL and looks up the corresponding task.
/// If the task is found, it returns it as JSON. If not, it returns a 404 Not Found status.
/// Axum automatically returns 400 Bad Request if the `id` cannot be parsed as a u64.
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, RustFlowError> {
    let store = state.tasks.0.read().await;

    store
        .tasks
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .map(Json)
        .ok_or(RustFlowError::NotFound(format!(
            "Task with id {} does not exist",
            id
        )))
}

async fn create(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> Result<(StatusCode, Json<Task>), RustFlowError> {
    // Acquire a write lock - exclusive access, blocks all other readers/writers
    let mut store = state.tasks.0.write().await;

    if store.tasks.iter().any(|x| payload.title == x.title) {
        return Err(RustFlowError::Conflict(format!(
            "Task with title {} already exists",
            payload.title
        )));
    }

    let task = Task {
        id: store.next_id,
        title: payload.title,
        description: payload.description,
        priority: payload.priority,
        status: TaskStatus::Pending,
    };

    store.next_id += 1;
    store.tasks.push(task.clone());
    println!(
        "[audit] Task {} created by client '{}'",
        task.id, client.name
    );

    Ok((StatusCode::CREATED, Json(task)))
}

async fn delete(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, RustFlowError> {
    let mut store = state.tasks.0.write().await;
    let len_before = store.tasks.len();

    store.tasks.retain(|task| task.id != id);

    if store.tasks.len() < len_before {
        println!("[audit] Task {id} deleted by client '{}'", client.name);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(RustFlowError::NotFound(format!(
            "Task with id {id} does not exist"
        )))
    }
}

async fn update(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> Result<Json<Task>, RustFlowError> {
    let mut store = state.tasks.0.write().await;

    if let Some(task) = store.tasks.iter_mut().find(|task| task.id == id) {
        task.title = payload.title;
        task.description = payload.description;
        task.priority = payload.priority;
        println!("[audit] Task {id} updated by client '{}'", client.name);
        Ok(Json(task.clone()))
    } else {
        Err(RustFlowError::NotFound(format!(
            "Task with id {} does not exist",
            id
        )))
    }
}

async fn change_status(
    Extension(client): Extension<AuthenticatedClient>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<TaskStatus>,
) -> Result<Json<Task>, RustFlowError> {
    let mut store = state.tasks.0.write().await;

    if let Some(task) = store.tasks.iter_mut().find(|task| task.id == id) {
        task.status = payload;
        println!(
            "[audit] Task {id} status changed by client '{}'",
            client.name
        );
        Ok(Json(task.clone()))
    } else {
        Err(RustFlowError::NotFound(format!(
            "Task with id {} does not exist",
            id
        )))
    }
}

// ── Router ──────────────────────────────────────────────────

pub fn router(state: AppState) -> Router<AppState> {
    // Public routes — no authentication required
    let public = rate_limited(
        Router::new()
            .route("/", get(list))
            .route("/{id}", get(get_one)),
        100,
        Duration::from_secs(10),
    );

    // Protected routes — require valid API key, identity injected
    let protected = rate_limited(
        Router::new()
            .route("/", post(create))
            .route("/{id}", put(update).delete(delete))
            .route("/{id}/status", patch(change_status))
            .layer(middleware::from_fn_with_state(state, require_api_key)),
        10,
        Duration::from_secs(10),
    );

    // Merge and apply logging to all routes
    public.merge(protected)
}
