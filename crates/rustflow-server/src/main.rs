mod extractors;
mod routes;
mod state;

use crate::state::AppState;
use axum::{routing::get, Router};
use rustflow_common::{
    APP_NAME, APP_VERSION
    ,
};
use tokio::net::TcpListener;

/// Root endpoint - a simple liveness message
async fn root() -> &'static str {
    "RustFlow is running!"
}

#[tokio::main]
async fn main() {
    let state: AppState = AppState::new();
    // Build the application router
    let app = Router::new()
        // Root liveness
        .route("/", get(root))
        // Health & config — merged at root level (no prefix)
        .merge(routes::health::router())
        // Domain routes — nested under /api/*
        .nest("/api/tasks", routes::tasks::router())
        .nest("/api/projects", routes::projects::router())
        .nest("/api/users", routes::users::router())
        // Provide state to ALL routes (merged and nested)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await.expect("Server error");
}
