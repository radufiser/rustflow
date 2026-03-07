use rustflow_common::{APP_NAME, APP_VERSION, DEFAULT_PORT, HealthStatus};

#[tokio::main]
async fn main() {
    println!("Starting {} port {} v{}", APP_NAME, DEFAULT_PORT, APP_VERSION);

    // Verify that shared types from rustflow_common are working
    let health = HealthStatus::default();
    println!(
        "Health Check: {}",
        serde_json::to_string_pretty(&health).unwrap()
    );

    println!("Workspace configured correctly!");
    println!();
    println!("Next up: Section 2.1 — we'll turn this into a real HTTP server with Axum.");
}
