//! Screen capture tools

use crate::error::PcControllerError;
use crate::platform::Platform;
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

/// Screen capture mode
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMode {
    Fullscreen,
    Window,
    Region,
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

    Ok(CallToolResult::success(vec![Content::text(format!("Screenshot captured: {} bytes", image_bytes.len()))]))
}
