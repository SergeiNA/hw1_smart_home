use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use crate::{api, state};

/// Start server (blocks until shutdown) - for production/daemon
pub async fn start_app(addr: SocketAddr) {
    let state = state::spawn_app_state();
    let app = api::routes::spawn_router(state).layer(TraceLayer::new_for_http());

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Spawn server in background on random port - for Rust tests
/// Returns the actual bound address
pub async fn spawn_test_server(addr: SocketAddr)  {
    let state = state::spawn_app_state();
    let app = api::routes::spawn_router(state).layer(TraceLayer::new_for_http());

    // Port 0 = OS assigns random available port
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
}