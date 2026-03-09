use axum::{middleware, routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

mod app_state;
mod auth;
mod comment;
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod research;

use app_state::AppState;
use config::AppConfig;

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
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env();
    let db_pool = db::connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    let state = AppState { db_pool, config };
    let app = Router::new()
        .route("/", get(|| async { "ResearchHub backend" }))
        .route("/api/health", get(health))
        .nest("/api/auth", auth::handler::routes())
        .nest(
            "/api/research",
            research::handler::routes().layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::auth::require_auth,
            )),
        )
        .nest(
            "/api/comments",
            comment::handler::routes().layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::auth::require_auth,
            )),
        )
        .with_state(state.clone());
    let addr: SocketAddr = format!("{}:{}", state.config.host, state.config.port)
        .parse()
        .expect("invalid host/port");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
