//! Error types for PC Controller MCP

use thiserror::Error;

/// Errors that can occur in PC Controller operations
#[derive(Error, Debug)]
pub enum PcControllerError {
    /// Platform-specific error
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// Window not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// Permission denied (e.g., macOS accessibility permissions)
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Screen capture failed
    #[error("Capture error: {0}")]
    CaptureError(String),

    /// Input simulation failed
    #[error("Input error: {0}")]
    InputError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Window manager error
    #[error("Window manager error: {0}")]
    WindowManagerError(String),
}

impl From<PcControllerError> for rmcp::ErrorData {
    fn from(err: PcControllerError) -> Self {
        rmcp::ErrorData {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: err.to_string().into(),
            data: None,
        }
    }
}
