use crate::extractors::{ValidatedJson, ValidatedQuery};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use rustflow_common::{CreateTask, Task, TaskFilter, TaskStatus};

/// GET /tasks?status=pending&priority=high
///
/// The ValidatedQuery extractor deserializes and validates query parameters into a TaskFilter struct.
async fn list(
    State(state): State<AppState>,
    ValidatedQuery(filter): ValidatedQuery<TaskFilter>,
) -> Json<Vec<Task>> {
    let state = state.tasks.0.read().await;

    let filtered: Vec<Task> = state
        .tasks
        .iter()
        .filter(|task| filter.status.as_ref().map_or(true, |s| &task.status == s))
        .filter(|task| filter.status.as_ref().map_or(true, |s| &task.status == s))
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
) -> Result<Json<Task>, StatusCode> {
    let store = state.tasks.0.read().await;

    store
        .tasks
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> (StatusCode, Json<Task>) {
    // Acquire a write lock - exclusive access, blocks all other readers/writers
    let mut store = state.tasks.0.write().await;

    let task = Task {
        id: store.next_id,
        title: payload.title,
        description: payload.description,
        priority: payload.priority,
        status: TaskStatus::Pending,
    };

    store.next_id += 1;
    store.tasks.push(task.clone());

    (StatusCode::CREATED, Json(task))
}

async fn delete(State(state): State<AppState>, Path(id): Path<u64>) -> StatusCode {
    let mut store = state.tasks.0.write().await;
    let len_before = store.tasks.len();

    store.tasks.retain(|task| task.id != id);

    if store.tasks.len() < len_before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut store = state.tasks.0.write().await;

    if let Some(task) = store.tasks.iter_mut().find(|task| task.id == id) {
        task.title = payload.title;
        task.description = payload.description;
        task.priority = payload.priority;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn change_status(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<TaskStatus>,
) -> Result<Json<Task>, StatusCode> {
    let mut store = state.tasks.0.write().await;

    if let Some(task) = store.tasks.iter_mut().find(|task| task.id == id) {
        task.status = payload;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete))
        .route("/{id}/status", patch(change_status))
}
