use std::collections::HashMap;
use rustflow_common::{AppConfig, Priority, Project, Task, TaskStatus, User};
use std::sync::Arc;
use tokio::sync::RwLock;

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
        let tasks = vec![
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

        Self { tasks, next_id }
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
            next_id: 1,
        }
    }
}

// ── Project State ──────────────────────────────────────────────
#[derive(Clone)]
pub struct UserState(pub Arc<RwLock<UserStore>>);
impl UserState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(UserStore::new())))
    }
}

pub struct UserStore {
    pub users: Vec<User>,
    pub next_id: u64,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: vec![],
            next_id: 1,
        }
    }
}

// Composite App State

#[derive(Clone)]
pub struct AppState {
    pub tasks: TaskState,
    pub projects: ProjectState,
    pub users: UserState,
    pub config: AppConfig,
    pub http_client: reqwest::Client,
    pub api_keys: HashMap<String, String>,
}

impl AppState {
    pub fn new() -> Self {
        let mut api_keys = HashMap::new();
        api_keys.insert(
            "rustflow-api-key-2026".to_string(),
            "default".to_string(),
        );
        api_keys.insert(
            "ci-bot-key-2026".to_string(),
            "ci-bot".to_string(),
        );
        api_keys.insert(
            "admin-key-2026".to_string(),
            "admin".to_string(),
        );
        Self {
            tasks: TaskState::new(),
            projects: ProjectState::new(),
            users: UserState::new(),
            config: AppConfig::default(),
            http_client: reqwest::Client::new(),
            api_keys
        }
    }
}
