use axum::{middleware, routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

mod app_state;
mod auth;
mod comment;
mod config;
mod dashboard;
mod db;
mod error;
mod middleware;
mod models;
mod research;
mod search;

use app_state::AppState;
use config::AppEnvironment;
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
    let config = AppConfig::from_env();
    let default_log = if config.environment == AppEnvironment::Production {
        "info"
    } else {
        "debug"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_log)),
        )
        .init();

    let db_pool = db::connect(&config)
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
        .nest(
            "/api/dashboard",
            dashboard::handler::routes().layer(middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::auth::require_auth,
            )),
        )
        .nest(
            "/api/search",
            search::handler::routes().layer(middleware::from_fn_with_state(
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
