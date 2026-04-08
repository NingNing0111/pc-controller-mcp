//! HTTP Streamable transport implementation

#[cfg(target_os = "macos")]
use crate::platform::{MacOSPlatform, Platform};
#[cfg(target_os = "windows")]
use crate::platform::{WindowsPlatform, Platform};
use crate::tools::PcController;
use axum::Router;
use axum::response::Json;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Run the MCP server over HTTP Streamable
pub async fn run<P: Platform + 'static>(
    platform: P,
    addr: SocketAddr,
    cors: bool,
) -> anyhow::Result<()> {
    let controller = PcController::new(platform);
    let ct = CancellationToken::new();

    let service = StreamableHttpService::new(
        move || Ok(controller.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default()
            .with_cancellation_token(ct.child_token()),
    );

    let mut app = Router::new()
        .nest_service("/mcp", service)
        .route("/health", axum::routing::get(handle_health));

    if cors {
        app = app.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    tracing::info!("Starting PC Controller MCP server over HTTP at {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await?;

    Ok(())
}

/// Run with platform auto-detection
#[cfg(target_os = "macos")]
pub async fn run_auto(addr: SocketAddr, cors: bool) -> anyhow::Result<()> {
    let platform = MacOSPlatform::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize macOS platform: {}", e))?;
    run(platform, addr, cors).await
}

#[cfg(target_os = "windows")]
pub async fn run_auto(addr: SocketAddr, cors: bool) -> anyhow::Result<()> {
    let platform = WindowsPlatform::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize Windows platform: {}", e))?;
    run(platform, addr, cors).await
}
