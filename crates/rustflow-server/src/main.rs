use axum::{Router, routing::get};
use rustflow_common::{APP_NAME, APP_VERSION, HealthStatus};
use tokio::net::TcpListener;

/// Root endpoint - a simple liveness message
async fn root() -> &'static str {
    "RustFlow is running!"
}

/// Health check endpoint - returns structured JSON about the service
async fn health() -> axum::Json<HealthStatus> {
    axum::Json(HealthStatus::default())
}

#[tokio::main]
async fn main() {
    // Build the application router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await.expect("Server error");
}
