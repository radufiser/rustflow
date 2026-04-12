use crate::state::AppState;
use axum::extract::{Path, State};
use axum::{
    routing::get,
    Json,
    Router,
};
use crate::errors::RustFlowError;
use tracing::Instrument;
// -------------- Types ------------

/// A user fetched from the external JSONPlaceholder API
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ExternalUser {
    id: u64,
    name: String,
    email: String,
    username: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EnrichedTask {
    id: u64,
    title: String,
    description: Option<String>,
    priority: String,
    status: String,
    assign_to: ExternalUser,
}

// --------- Handlers ------------------
/// GET /api/enrichment/tasks/:task_id/user/:user_id
///
/// Fetches a task from local state and a user from an external API,
/// then combines them into an enriched response.
///
/// This demonstrates:
/// 1. Accessing local state (task store)
/// 2. Making an outbound HTTP call (reqwest)
/// 3. Combining data from multiple sources
/// 4. Handling errors from external services
#[tracing::instrument(name = "enrichment.enrich", skip(state))]
async fn enrich_task(
    State(state): State<AppState>,
    Path((task_id, user_id)): Path<(u64, u64)>,
) -> Result<Json<EnrichedTask>, RustFlowError> {
    // Step 1: Find the task in the local statement
    //
    // Important: We scope the lock so it is dropped before the HTTP call.
    // if we held the read lock across the network request, we'd block All other handlers for the
    // entire duration of the external call.
    let task = {
        let store = state.tasks.0.read().await;
        store
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .cloned()
            .ok_or(RustFlowError::NotFound(format!("Task with id {task_id} not found")))?
    };
    // <- read lock is dropped here (end of the block)

    // Step 2: Fetch user from external service
    let url = format!("https://jsonplaceholder.typicode.com/users/{}", user_id);

    let user: ExternalUser = async {
        state
            .http_client
            .get(&url)
            // Network or request error (e.g., DNS, connection, timeout)
            .send()
            .await
            .map_err(|e| RustFlowError::ExternalService(e.to_string()))?
            // HTTP status error (e.g., 404, 500)
            .error_for_status()
            .map_err(|e| RustFlowError::ExternalService(e.to_string()))?
            // JSON deserialization error (e.g., invalid/mismatched JSON)
            .json()
            .await
            .map_err(|e| RustFlowError::ExternalService(e.to_string()))
    }
    .instrument(tracing::info_span!("http.get", url = %url))
    .await?;

    // Step 3: Combine into a rich response

    let enriched = EnrichedTask {
        id: task_id,
        title: task.title,
        description: task.description,
        priority: format!("{:?}", task.priority),
        status: format!("{:?}", task.status),
        assign_to: user,
    };
    Ok(Json(enriched))
}

/// GET /api/enrichment/external-users
///
/// A simpler example: just proxy an external API call and return the result.
/// Useful for testing that the HTTP client works.
#[tracing::instrument(name = "external_users.list", skip(state))]
async fn list_external_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExternalUser>>, RustFlowError> {
    let users: Vec<ExternalUser> = state
        .http_client
        .get("https://jsonplaceholder.typicode.com/users")
        .send()
        .await
        .map_err(|e|  RustFlowError::ExternalService(e.to_string()))?
        .json()
        .await
        .map_err(|e|  RustFlowError::ExternalService(e.to_string()))?;

    Ok(Json(users))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks/{task_id}/user/{user_id}", get(enrich_task))
        .route("/external-users", get(list_external_users))
}
