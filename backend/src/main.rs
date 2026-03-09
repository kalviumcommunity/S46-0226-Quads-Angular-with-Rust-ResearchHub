use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(|| async { "ResearchHub backend" }));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
