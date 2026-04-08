//! Screen capture tools

use crate::error::PcControllerError;
use crate::platform::Platform;
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Screen capture mode
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMode {
    Fullscreen,
    Window,
    Region,
}

/// Screen capture output format
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Return file path to PNG image
    File,
    /// Return base64 encoded image data
    Base64,
    /// Return raw bytes (for backward compatibility)
    Bytes,
}

/// Arguments for screen capture
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CaptureScreenArgs {
    /// Capture mode: fullscreen, window, or region
    #[serde(default)]
    pub mode: Option<CaptureMode>,

    /// Window ID for window mode
    #[serde(default)]
    pub window_id: Option<String>,

    /// Region for region mode: [x, y, width, height]
    #[serde(default)]
    pub region: Option<Vec<i32>>,

    /// Display ID for multi-monitor
    #[serde(default)]
    pub display_id: Option<u32>,

    /// Output format: file (default), base64, bytes
    #[serde(default)]
    pub format: Option<OutputFormat>,
}

/// Save screenshot to temp file and return the path
fn save_to_temp_file(image_bytes: &[u8], prefix: &str) -> Result<PathBuf, PcControllerError> {
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to get timestamp: {}", e)))?
        .as_millis();

    let filename = format!("{}_{}.png", prefix, timestamp);
    let file_path = temp_dir.join(&filename);

    fs::write(&file_path, image_bytes)
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to write temp file: {}", e)))?;

    Ok(file_path)
}

/// Execute screen capture
pub fn capture_screen<P: Platform>(
    platform: &P,
    args: &CaptureScreenArgs,
) -> Result<CallToolResult, PcControllerError> {
    let mode = args.mode.clone().unwrap_or(CaptureMode::Fullscreen);

    let image_bytes = match mode {
        CaptureMode::Fullscreen => {
            platform.capture_fullscreen(args.display_id)?
        }
        CaptureMode::Window => {
            let window_id = args.window_id.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("window_id required for window mode".to_string()))?;
            platform.capture_window(window_id)?
        }
        CaptureMode::Region => {
            let region = args.region.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("region required for region mode".to_string()))?;

            if region.len() != 4 {
                return Err(PcControllerError::InvalidArguments("region must be [x, y, width, height]".to_string()));
            }

            platform.capture_region(region[0], region[1], region[2] as u32, region[3] as u32)?
        }
    };

    let format = args.format.as_ref().unwrap_or(&OutputFormat::File);

    let result = match format {
        OutputFormat::File => {
            let path = save_to_temp_file(&image_bytes, "screenshot")?;
            serde_json::json!({
                "path": path.to_string_lossy(),
                "size": image_bytes.len()
            }).to_string()
        }
        OutputFormat::Base64 => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let base64 = STANDARD.encode(&image_bytes);
            serde_json::json!({
                "base64": base64,
                "size": image_bytes.len()
            }).to_string()
        }
        OutputFormat::Bytes => {
            format!("Screenshot captured: {} bytes", image_bytes.len())
        }
    };

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
