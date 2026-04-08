//! PC Controller MCP Server Library
//!
//! A Model Context Protocol server for PC control capabilities including:
//! - Screen capture (fullscreen, window, region)
//! - Window management (list, focus)
//! - Input simulation (keyboard, mouse)

pub mod error;
pub mod platform;
pub mod tools;
pub mod protocol;

pub use error::PcControllerError;
pub use platform::{Platform, WindowInfo, WindowBounds, InputAction, KeyModifier, MouseButton};
pub use tools::PcController;

#[cfg(target_os = "macos")]
pub use platform::macos::MacOSPlatform;

#[cfg(target_os = "windows")]
pub use platform::windows::WindowsPlatform;
