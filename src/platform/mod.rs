//! Platform abstraction layer
//!
//! Provides a unified interface for platform-specific operations

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub use macos::MacOSPlatform;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform;

use crate::error::PcControllerError;

/// Unified platform trait
pub trait Platform: Send + Sync {
    // Window management
    fn list_windows(&self) -> Result<Vec<WindowInfo>, PcControllerError>;
    fn focus_window(&self, window_id: &str) -> Result<(), PcControllerError>;

    // Screen capture
    fn capture_fullscreen(&self, display_id: Option<u32>) -> Result<Vec<u8>, PcControllerError>;
    fn capture_window(&self, window_id: &str) -> Result<Vec<u8>, PcControllerError>;
    fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>, PcControllerError>;

    // Input simulation
    fn keyboard_type(&self, text: &str) -> Result<(), PcControllerError>;
    fn keyboard_key(&self, key: &str, action: InputAction) -> Result<(), PcControllerError>;
    fn keyboard_combo(&self, keys: &[&str], modifiers: &[KeyModifier]) -> Result<(), PcControllerError>;
    fn mouse_move(&self, x: i32, y: i32) -> Result<(), PcControllerError>;
    fn mouse_click(&self, button: MouseButton, action: InputAction) -> Result<(), PcControllerError>;
    fn mouse_scroll(&self, delta_x: i32, delta_y: i32) -> Result<(), PcControllerError>;
}

/// Window information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowInfo {
    pub window_id: String,
    pub title: String,
    pub app_name: String,
    pub process_id: u32,
    pub is_minimized: bool,
    pub is_visible: bool,
    pub display_id: u32,
    pub bounds: WindowBounds,
}

/// Window bounds (position and size)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Input action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Press,
    Release,
    Type,
}

/// Key modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Cmd,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl std::fmt::Display for KeyModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyModifier::Ctrl => write!(f, "ctrl"),
            KeyModifier::Alt => write!(f, "alt"),
            KeyModifier::Shift => write!(f, "shift"),
            KeyModifier::Cmd => write!(f, "cmd"),
        }
    }
}

impl std::fmt::Display for MouseButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseButton::Left => write!(f, "left"),
            MouseButton::Right => write!(f, "right"),
            MouseButton::Middle => write!(f, "middle"),
        }
    }
}
