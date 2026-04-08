//! macOS platform implementation

use crate::error::PcControllerError;
use crate::platform::{
    InputAction, KeyModifier, MouseButton, Platform, WindowBounds, WindowInfo,
};
use enigo::{
    Axis, Button, Coordinate, Direction, Enigo, Keyboard, Key, Mouse, Settings,
};
use std::sync::{Arc, Mutex};
use xcap::Monitor;

#[derive(Clone)]
pub struct MacOSPlatform {
    enigo: Arc<Mutex<Enigo>>,
}

impl MacOSPlatform {
    pub fn new() -> Result<Self, PcControllerError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| PcControllerError::PlatformError(format!("Failed to create enigo: {}", e)))?;

        Ok(Self {
            enigo: Arc::new(Mutex::new(enigo)),
        })
    }

    fn get_monitors(&self) -> Result<Vec<Monitor>, PcControllerError> {
        Monitor::all()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to get monitors: {}", e)))
    }
}

impl Platform for MacOSPlatform {
    fn list_windows(&self) -> Result<Vec<WindowInfo>, PcControllerError> {
        use std::process::Command;

        let script = r#"
            tell application "System Events"
                set windowList to {}
                set processList to every process whose windows length > 0
                repeat with theProcess in processList
                    set processName to name of theProcess
                    try
                        set windowRefs to windows of theProcess
                        repeat with theWindow in windowRefs
                            set windowTitle to name of theWindow
                            set windowPos to position of theWindow
                            set windowSize to size of theWindow
                            set minimized to value of attribute "AXMinimized" of theWindow
                            set visible to value of attribute "AXVisible" of theWindow

                            set windowInfo to {processName, windowTitle, item 1 of windowPos, item 2 of windowPos, item 1 of windowSize, item 2 of windowSize, minimized, visible}
                            set end of windowList to windowInfo
                        end repeat
                    end try
                end repeat
            end tell
            return windowList
        "#;

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| PcControllerError::PlatformError(format!("Failed to execute osascript: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut windows = Vec::new();

        for (idx, line) in stdout.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 7 {
                windows.push(WindowInfo {
                    window_id: format!("window_{}", idx),
                    title: parts.get(1).unwrap_or(&"Unknown").to_string(),
                    app_name: parts.get(0).unwrap_or(&"Unknown").to_string(),
                    process_id: 0,
                    is_minimized: parts.get(6).map(|s| s.trim() == "true").unwrap_or(false),
                    is_visible: parts.get(7).map(|s| s.trim() == "true").unwrap_or(true),
                    display_id: 0,
                    bounds: WindowBounds {
                        x: parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0),
                        y: parts.get(3).and_then(|s| s.trim().parse().ok()).unwrap_or(0),
                        width: parts.get(4).and_then(|s| s.trim().parse().ok()).unwrap_or(800),
                        height: parts.get(5).and_then(|s| s.trim().parse().ok()).unwrap_or(600),
                    },
                });
            }
        }

        Ok(windows)
    }

    fn focus_window(&self, window_id: &str) -> Result<(), PcControllerError> {
        use std::process::Command;

        let script = format!(
            r#"tell application "System Events" to tell process "{}" to set frontmost to true"#,
            window_id.replace('"', "\\\"")
        );

        Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| PcControllerError::WindowManagerError(format!("Failed to focus window: {}", e)))?;

        Ok(())
    }

    fn capture_fullscreen(&self, _display_id: Option<u32>) -> Result<Vec<u8>, PcControllerError> {
        let monitors = self.get_monitors()?;

        if monitors.is_empty() {
            return Err(PcControllerError::CaptureError("No monitors found".to_string()));
        }

        let monitor = &monitors[0];
        let image = monitor
            .capture_image()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to capture screen: {}", e)))?;

        // Convert to PNG using image crate
        use image::ImageEncoder;
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        ).map_err(|e| PcControllerError::CaptureError(format!("Failed to encode PNG: {}", e)))?;

        Ok(buffer)
    }

    fn capture_window(&self, _window_id: &str) -> Result<Vec<u8>, PcControllerError> {
        self.capture_fullscreen(None)
    }

    fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>, PcControllerError> {
        let monitors = self.get_monitors()?;
        if monitors.is_empty() {
            return Err(PcControllerError::CaptureError("No monitors found".to_string()));
        }

        let monitor = &monitors[0];
        let image = monitor
            .capture_image()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to capture screen: {}", e)))?;

        let cropped = image::imageops::crop_imm(&image, x as u32, y as u32, width, height).to_image();

        use image::ImageEncoder;
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.write_image(cropped.as_raw(), width, height, image::ExtendedColorType::Rgba8)
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to encode PNG: {}", e)))?;

        Ok(buffer)
    }

    fn keyboard_type(&self, text: &str) -> Result<(), PcControllerError> {
        let mut enigo = self.enigo.lock().unwrap();
        enigo.text(text)
            .map_err(|e| PcControllerError::InputError(format!("Failed to type text: {}", e)))?;
        Ok(())
    }

    fn keyboard_key(&self, key: &str, action: InputAction) -> Result<(), PcControllerError> {
        let key = match key.to_lowercase().as_str() {
            "a" => Key::Unicode('a'),
            "b" => Key::Unicode('b'),
            "c" => Key::Unicode('c'),
            "d" => Key::Unicode('d'),
            "e" => Key::Unicode('e'),
            "f" => Key::Unicode('f'),
            "g" => Key::Unicode('g'),
            "h" => Key::Unicode('h'),
            "i" => Key::Unicode('i'),
            "j" => Key::Unicode('j'),
            "k" => Key::Unicode('k'),
            "l" => Key::Unicode('l'),
            "m" => Key::Unicode('m'),
            "n" => Key::Unicode('n'),
            "o" => Key::Unicode('o'),
            "p" => Key::Unicode('p'),
            "q" => Key::Unicode('q'),
            "r" => Key::Unicode('r'),
            "s" => Key::Unicode('s'),
            "t" => Key::Unicode('t'),
            "u" => Key::Unicode('u'),
            "v" => Key::Unicode('v'),
            "w" => Key::Unicode('w'),
            "x" => Key::Unicode('x'),
            "y" => Key::Unicode('y'),
            "z" => Key::Unicode('z'),
            "0" => Key::Unicode('0'),
            "1" => Key::Unicode('1'),
            "2" => Key::Unicode('2'),
            "3" => Key::Unicode('3'),
            "4" => Key::Unicode('4'),
            "5" => Key::Unicode('5'),
            "6" => Key::Unicode('6'),
            "7" => Key::Unicode('7'),
            "8" => Key::Unicode('8'),
            "9" => Key::Unicode('9'),
            "return" | "enter" => Key::Return,
            "space" => Key::Space,
            "tab" => Key::Tab,
            "escape" | "esc" => Key::Escape,
            "backspace" => Key::Backspace,
            "delete" => Key::Delete,
            "up" => Key::UpArrow,
            "down" => Key::DownArrow,
            "left" => Key::LeftArrow,
            "right" => Key::RightArrow,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            _ => return Err(PcControllerError::InputError(format!("Unknown key: {}", key))),
        };

        let mut enigo = self.enigo.lock().unwrap();
        let direction = match action {
            InputAction::Press | InputAction::Type => Direction::Press,
            InputAction::Release => Direction::Release,
        };

        enigo.key(key, direction)
            .map_err(|e| PcControllerError::InputError(format!("Failed to send key: {}", e)))?;
        Ok(())
    }

    fn keyboard_combo(&self, keys: &[&str], modifiers: &[KeyModifier]) -> Result<(), PcControllerError> {
        let mut enigo = self.enigo.lock().unwrap();

        for modifier in modifiers {
            let key = match modifier {
                KeyModifier::Ctrl => Key::Control,
                KeyModifier::Alt | KeyModifier::Cmd => Key::Option,
                KeyModifier::Shift => Key::Shift,
            };
            enigo.key(key, Direction::Press)
                .map_err(|e| PcControllerError::InputError(format!("Failed to press modifier: {}", e)))?;
        }

        for key_str in keys {
            let key = match key_str.to_lowercase().as_str() {
                "a" => Key::Unicode('a'),
                "c" => Key::Unicode('c'),
                "v" => Key::Unicode('v'),
                "x" => Key::Unicode('x'),
                "z" => Key::Unicode('z'),
                _ => continue,
            };
            enigo.key(key, Direction::Press)
                .map_err(|e| PcControllerError::InputError(format!("Failed to press key: {}", e)))?;
        }

        for key_str in keys.iter().rev() {
            let key = match key_str.to_lowercase().as_str() {
                "a" => Key::Unicode('a'),
                "c" => Key::Unicode('c'),
                "v" => Key::Unicode('v'),
                "x" => Key::Unicode('x'),
                "z" => Key::Unicode('z'),
                _ => continue,
            };
            enigo.key(key, Direction::Release)
                .map_err(|e| PcControllerError::InputError(format!("Failed to release key: {}", e)))?;
        }

        for modifier in modifiers.iter().rev() {
            let key = match modifier {
                KeyModifier::Ctrl => Key::Control,
                KeyModifier::Alt | KeyModifier::Cmd => Key::Option,
                KeyModifier::Shift => Key::Shift,
            };
            enigo.key(key, Direction::Release)
                .map_err(|e| PcControllerError::InputError(format!("Failed to release modifier: {}", e)))?;
        }

        Ok(())
    }

    fn mouse_move(&self, x: i32, y: i32) -> Result<(), PcControllerError> {
        let mut enigo = self.enigo.lock().unwrap();
        enigo.move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| PcControllerError::InputError(format!("Failed to move mouse: {}", e)))?;
        Ok(())
    }

    fn mouse_click(&self, button: MouseButton, action: InputAction) -> Result<(), PcControllerError> {
        let enigo_button = match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
        };

        let direction = match action {
            InputAction::Press => Direction::Press,
            InputAction::Release => Direction::Release,
            InputAction::Type => Direction::Click,
        };

        let mut enigo = self.enigo.lock().unwrap();
        enigo.button(enigo_button, direction)
            .map_err(|e| PcControllerError::InputError(format!("Failed to click mouse: {}", e)))?;
        Ok(())
    }

    fn mouse_scroll(&self, delta_x: i32, delta_y: i32) -> Result<(), PcControllerError> {
        let mut enigo = self.enigo.lock().unwrap();

        if delta_x != 0 {
            enigo.scroll(delta_x, Axis::Horizontal)
                .map_err(|e| PcControllerError::InputError(format!("Failed to scroll: {}", e)))?;
        }

        if delta_y != 0 {
            enigo.scroll(delta_y, Axis::Vertical)
                .map_err(|e| PcControllerError::InputError(format!("Failed to scroll: {}", e)))?;
        }

        Ok(())
    }
}
