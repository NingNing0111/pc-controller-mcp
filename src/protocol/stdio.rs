//! STDIO transport implementation

use crate::platform::{MacOSPlatform, Platform};
use crate::tools::PcController;
use rmcp::ServiceExt;

/// Run the MCP server over STDIO
pub async fn run<P: Platform + 'static>(platform: P) -> anyhow::Result<()> {
    let controller = PcController::new(platform);

    tracing::info!("Starting PC Controller MCP server over STDIO");

    let service = controller
        .serve(rmcp::transport::stdio())
        .await
        .inspect_err(|e| tracing::error!("Server error: {:?}", e))?;

    service.waiting().await?;

    Ok(())
}

/// Run with platform auto-detection
pub async fn run_auto() -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let platform = MacOSPlatform::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize macOS platform: {}", e))?;
        run(platform).await
    }

    #[cfg(target_os = "windows")]
    {
        Err(anyhow::anyhow!("Windows platform not yet implemented"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}
