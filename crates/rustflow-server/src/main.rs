mod errors;
mod extractors;
mod routes;
mod state;

use crate::state::AppState;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::Router;
use rustflow_common::{APP_NAME, APP_VERSION};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let state: AppState = AppState::new();

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-api-key"),
        ])
        .max_age(Duration::from_secs(3600));

    // Build the application router
    let app = Router::new()
        // Serve the dashboard at root
        .route_service(
            "/",
            ServeFile::new("crates/rustflow-server/static/index.html"),
        )
        // Serve static files from /static/* — automatically serves index.html for directory requests
        .nest_service(
            "/static",
            ServeDir::new("crates/rustflow-server/static").append_index_html_on_directories(true),
        )
        .route_service(
            "/favicon.ico",
            ServeFile::new("crates/rustflow-server/static/favicon.ico"),
        )
        // Health & config — merged at root level (no prefix)
        .merge(routes::health::router())
        // Domain routes — nested under /api/*
        .nest("/api/tasks", routes::tasks::router(state.clone()))
        .nest("/api/projects", routes::projects::router(state.clone()))
        .nest("/api/users", routes::users::router(state.clone()))
        // Enrichment — combines local data with external API calls
        .nest("/api/enrichment", routes::enrichment::router())
        // Provide state to ALL routes (merged and nested)
        .with_state(state)
        // CORS — outermost layer, runs first on every request
        .layer(cors);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    println!("{} v{} listening on {}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await.expect("Server error");
}
