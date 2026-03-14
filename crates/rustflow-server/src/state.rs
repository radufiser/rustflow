use std::sync::Arc;
use tokio::sync::RwLock;
use rustflow_common::{AppConfig, Priority, Project, Task, TaskStatus};

// ── Task State ──────────────────────────────────────────────

/// State for task management. Independently lockable.
#[derive(Clone)]
pub struct TaskState(pub Arc<RwLock<TaskStore>>);

impl TaskState {
    pub fn new() -> TaskState {
        Self(Arc::new(RwLock::new(TaskStore::new())))
    }
}
pub struct TaskStore {
    pub tasks: Vec<Task>,
    pub next_id: u64,
}

impl TaskStore {
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

// ── Project State ──────────────────────────────────────────────

/// State for task management. Independently lockable.
#[derive(Clone)]
pub struct ProjectState(pub Arc<RwLock<ProjectStore>>);

impl ProjectState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ProjectStore::new())))
    }
}

pub struct ProjectStore {
    pub projects: Vec<Project>,
    pub next_id: u64,
}
impl ProjectStore {
    pub fn new() -> Self {
        Self {
            projects: vec![],
            next_id: 0,
        }

    }
}

// Composite App State

#[derive(Clone)]
pub struct AppState {
    pub tasks: TaskState,
    pub projects: ProjectState,
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: TaskState::new(),
            projects: ProjectState::new(),
            config: AppConfig::default(),
        }
    }
}