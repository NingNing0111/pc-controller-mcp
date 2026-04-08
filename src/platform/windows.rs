//! Windows platform implementation

use crate::error::PcControllerError;
use crate::platform::{
    InputAction, KeyModifier, MouseButton, Platform, WindowBounds, WindowInfo,
};
use enigo::{
    Axis, Button, Coordinate, Direction, Enigo, Keyboard, Key, Mouse, Settings,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WindowsPlatform {
    enigo: Arc<Mutex<Enigo>>,
}

impl WindowsPlatform {
    pub fn new() -> Result<Self, PcControllerError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| PcControllerError::PlatformError(format!("Failed to create enigo: {}", e)))?;

        Ok(Self {
            enigo: Arc::new(Mutex::new(enigo)),
        })
    }
}

impl Platform for WindowsPlatform {
    fn list_windows(&self) -> Result<Vec<WindowInfo>, PcControllerError> {
        use std::process::Command;

        let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Diagnostics;
using System.Collections.Generic;

public class WindowHelper {
    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    private static extern int GetWindowTextLength(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    private static extern bool IsIconic(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern IntPtr GetShellWindow();

    [DllImport("user32.dll")]
    private static extern int GetWindowLong(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    public static string[] GetWindows() {
        var windows = new List<string>();
        var shellWindow = GetShellWindow();
        var visibleWindows = new List<IntPtr>();

        EnumWindows((hWnd, lParam) => {
            if (hWnd == shellWindow) return true;
            if (!IsWindowVisible(hWnd)) return true;
            int style = GetWindowLong(hWnd, -16);
            if (style == 0) return true;

            int length = GetWindowTextLength(hWnd);
            if (length == 0) return true;

            var sb = new StringBuilder(length + 1);
            GetWindowText(hWnd, sb, sb.Capacity);
            string title = sb.ToString();

            if (string.IsNullOrWhiteSpace(title)) return true;

            uint processId;
            GetWindowThreadProcessId(hWnd, out processId);

            RECT rect;
            GetWindowRect(hWnd, out rect);

            bool minimized = IsIconic(hWnd);

            // Format: hwnd|title|processId|x|y|width|height|minimized|processName
            string processName = "Unknown";
            try {
                Process p = Process.GetProcessById((int)processId);
                processName = p.ProcessName;
            } catch {}

            windows.Add(string.Format("{0}|{1}|{2}|{3}|{4}|{5}|{6}|{7}|{8}",
                hWnd, title.Replace("|", " ").Replace("\n", " "), processId, 
                rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top, 
                minimized, processName));

            return true;
        }, IntPtr.Zero);

        return windows.ToArray();
    }
}
"@

$windows = [WindowHelper]::GetWindows()
foreach ($w in $windows) { Write-Output $w }
"#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|e| PcControllerError::PlatformError(format!("Failed to execute PowerShell: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !stderr.is_empty() {
            tracing::debug!("PowerShell stderr: {}", stderr);
        }
        
        tracing::debug!("PowerShell stdout: {}", stdout);

        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut windows = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();

            if parts.len() >= 9 {
                windows.push(WindowInfo {
                    window_id: parts[0].to_string(),
                    title: parts[1].to_string(),
                    app_name: parts[8].to_string(),
                    process_id: parts[2].parse().unwrap_or(0),
                    is_minimized: parts[7] == "True",
                    is_visible: true,
                    display_id: 0,
                    bounds: WindowBounds {
                        x: parts[3].parse().unwrap_or(0),
                        y: parts[4].parse().unwrap_or(0),
                        width: parts[5].parse().unwrap_or(800),
                        height: parts[6].parse().unwrap_or(600),
                    },
                });
            }
        }

        tracing::debug!("Found {} windows", windows.len());
        Ok(windows)
    }

    fn focus_window(&self, window_id: &str) -> Result<(), PcControllerError> {
        use std::process::Command;

        let hwnd: isize = window_id
            .parse()
            .map_err(|_| PcControllerError::InvalidArguments("Invalid window ID".to_string()))?;

        let script = format!(
            r#"Add-Type @"
using System;
using System.Runtime.InteropServices;
public class WindowFocus {{
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    public const int SW_RESTORE = 9;
}}
"@
$hwnd = [IntPtr]::new({})
[WindowFocus]::ShowWindow($hwnd, [WindowFocus]::SW_RESTORE)
[WindowFocus]::SetForegroundWindow($hwnd)
"#,
            hwnd
        );

        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(|e| PcControllerError::WindowManagerError(format!("Failed to focus window: {}", e)))?;

        Ok(())
    }

    fn capture_fullscreen(&self, _display_id: Option<u32>) -> Result<Vec<u8>, PcControllerError> {
        use image::ImageEncoder;
        use xcap::Monitor;

        let monitors = Monitor::all()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to get monitors: {}", e)))?;

        if monitors.is_empty() {
            return Err(PcControllerError::CaptureError("No monitors found".to_string()));
        }

        let monitor = &monitors[0];
        let image = monitor
            .capture_image()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to capture screen: {}", e)))?;

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
        use image::ImageEncoder;

        let monitors = xcap::Monitor::all()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to get monitors: {}", e)))?;

        if monitors.is_empty() {
            return Err(PcControllerError::CaptureError("No monitors found".to_string()));
        }

        let monitor = &monitors[0];
        let image = monitor
            .capture_image()
            .map_err(|e| PcControllerError::CaptureError(format!("Failed to capture screen: {}", e)))?;

        let cropped = image::imageops::crop_imm(&image, x as u32, y as u32, width, height).to_image();

        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.write_image(
            cropped.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        ).map_err(|e| PcControllerError::CaptureError(format!("Failed to encode PNG: {}", e)))?;

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
                KeyModifier::Alt => Key::Alt,
                KeyModifier::Shift => Key::Shift,
                KeyModifier::Cmd => Key::Meta,
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
                KeyModifier::Alt => Key::Alt,
                KeyModifier::Shift => Key::Shift,
                KeyModifier::Cmd => Key::Meta,
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
