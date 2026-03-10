use std::sync::Arc;
use tokio::sync::RwLock;
use rustflow_common::{Priority, Task, TaskStatus};

/// Shared application state. Wrapped in `Arc` so it can be cheaply clones across handlers invocations
/// (Axum clones state per request
pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub tasks: Vec<Task>,
    pub next_id: u64,
}

impl AppState {
    pub fn new() -> Self {

           let  tasks =  vec![
                Task {
                    id: 1,
                    title: "Set up workspace".into(),
                    description: Some("Initialize the Cargo workspace".into()),
                    priority: Priority::High,
                    status: TaskStatus::Done,
                },
                Task {
                    id: 2,
                    title: "Add Axum".into(),
                    description: Some("Create the HTTP server".into()),
                    priority: Priority::High,
                    status: TaskStatus::InProgress,
                },
                Task {
                    id: 3,
                    title: "Write extractors lesson".into(),
                    description: None,
                    priority: Priority::Medium,
                    status: TaskStatus::Pending,
                },
            ];
        let next_id = tasks.len() as u64 + 1;

        Self {
            tasks,
            next_id
        }
    }
}