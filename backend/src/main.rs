use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "researchhub-backend",
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(|| async { "ResearchHub backend" }))
        .route("/api/health", get(health));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
