mod extractors;
mod state;

use crate::extractors::{ValidatedJson, ValidatedQuery};
use crate::state::{AppState, SharedState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::patch;
use axum::{Json, Router, routing::get};
use rustflow_common::{
    APP_NAME, APP_VERSION, CreateTask, HealthStatus, Task, TaskFilter, TaskStatus,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

/// Root endpoint - a simple liveness message
async fn root() -> &'static str {
    "RustFlow is running!"
}

/// Health check endpoint - returns structured JSON about the service
async fn health() -> Json<HealthStatus> {
    Json(HealthStatus::default())
}

/// GET /tasks?status=pending&priority=high
///
/// The ValidatedQuery extractor deserializes and validates query parameters into a TaskFilter struct.
async fn list_tasks(
    State(state): State<SharedState>,
    ValidatedQuery(filter): ValidatedQuery<TaskFilter>,
) -> Json<Vec<Task>> {
    let state = state.read().await;

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
async fn get_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, StatusCode> {
    let state = state.read().await;

    state
        .tasks
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_task(
    State(state): State<SharedState>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> (StatusCode, Json<Task>) {
    // Acquire a write lock - exclusive access, blocks all other readers/writers
    let mut state = state.write().await;

    let task = Task {
        id: state.next_id,
        title: payload.title,
        description: payload.description,
        priority: payload.priority,
        status: TaskStatus::Pending,
    };

    state.next_id += 1;
    state.tasks.push(task.clone());

    tokio::time::sleep(Duration::from_secs(10)).await;
    (StatusCode::CREATED, Json(task))
}

async fn delete_task(State(state): State<SharedState>, Path(id): Path<u64>) -> StatusCode {
    let mut state = state.write().await;
    let len_before = state.tasks.len();

    state.tasks.retain(|task| task.id != id);

    if state.tasks.len() < len_before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn update_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    ValidatedJson(payload): ValidatedJson<CreateTask>,
) -> Result<Json<Task>, StatusCode> {
    let mut state = state.write().await;

    if let Some(task) = state.tasks.iter_mut().find(|task| task.id == id) {
        task.title = payload.title;
        task.description = payload.description;
        task.priority = payload.priority;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn change_task_status(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(payload): Json<TaskStatus>,
) -> Result<Json<Task>, StatusCode> {
    let mut state = state.write().await;

    if let Some(task) = state.tasks.iter_mut().find(|task| task.id == id) {
        task.status = payload;
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(RwLock::new(AppState::new()));
    // Build the application router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).put(update_task).delete(delete_task))
        .route("/tasks/{id}/status", patch(change_task_status))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await.expect("Server error");
}
