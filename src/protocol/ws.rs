//! WebSocket transport implementation

use crate::platform::MacOSPlatform;
use crate::tools::PcController;
use axum::{extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State}, Router};
use futures::{SinkExt, StreamExt};
use http::Response;
use std::net::SocketAddr;
use std::sync::Arc;

async fn handle_websocket(
    socket: WebSocket,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received WebSocket message: {}", text);
                let response = format!("MCP WebSocket echo: {}", text);
                if let Err(e) = ws_sender.send(Message::Text(response.into())).await {
                    tracing::error!("WebSocket send error: {}", e);
                    break;
                }
            }
            Ok(Message::Binary(data)) => {
                tracing::debug!("Received {} bytes", data.len());
                if let Err(e) = ws_sender.send(Message::Binary(data)).await {
                    tracing::error!("WebSocket send error: {}", e);
                    break;
                }
            }
            Ok(Message::Ping(data)) => {
                if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                    tracing::error!("WebSocket send error: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

async fn handle_upgrade(
    ws: WebSocketUpgrade,
    State(_handler): State<Arc<PcController<MacOSPlatform>>>,
) -> Response<axum::body::Body> {
    ws.on_upgrade(|socket| handle_websocket(socket))
}

/// Run the MCP server over WebSocket
pub async fn run(controller: PcController<MacOSPlatform>, addr: SocketAddr) -> anyhow::Result<()> {
    let handler: Arc<PcController<MacOSPlatform>> = Arc::new(controller);

    let app = Router::new()
        .route("/ws", axum::routing::get(handle_upgrade))
        .with_state(handler.clone());

    tracing::info!("Starting PC Controller MCP server over WebSocket at {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Run with platform auto-detection
pub async fn run_auto(addr: SocketAddr) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let platform = MacOSPlatform::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize macOS platform: {}", e))?;
        let controller = PcController::new(platform);
        run(controller, addr).await
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(anyhow::anyhow!("Only macOS is supported currently"))
    }
}
