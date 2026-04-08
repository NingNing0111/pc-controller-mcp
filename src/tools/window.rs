//! Window management tools

use crate::error::PcControllerError;
use crate::platform::Platform;
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

/// Arguments for focusing a window
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FocusWindowArgs {
    /// Window ID to focus
    pub window_id: String,
}

/// List all visible windows
pub fn list_windows<P: Platform>(
    platform: &P,
) -> Result<CallToolResult, PcControllerError> {
    let windows = platform.list_windows()?;

    let json = serde_json::to_string_pretty(&windows)
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to serialize windows: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Focus a specific window by ID
pub fn focus_window<P: Platform>(
    platform: &P,
    args: &FocusWindowArgs,
) -> Result<CallToolResult, PcControllerError> {
    platform.focus_window(&args.window_id)?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Window {} focused successfully",
        args.window_id
    ))]))
}
