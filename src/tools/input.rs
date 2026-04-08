//! Input simulation tools

use crate::error::PcControllerError;
use crate::platform::{InputAction, KeyModifier, MouseButton, Platform};
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

/// Input type
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Key,
    Text,
    Combo,
}

/// Keyboard action
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardAction {
    Press,
    Release,
    Type,
}

/// Mouse action
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MouseAction {
    Move,
    Click,
    DoubleClick,
    RightClick,
    Scroll,
    Drag,
}

/// Mouse button
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MouseButtonType {
    Left,
    Right,
    Middle,
}

/// Arguments for keyboard input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct KeyboardInputArgs {
    /// Input type: key, text, or combo
    #[serde(default)]
    pub input_type: Option<InputType>,

    /// Key code for key type
    #[serde(default)]
    pub key: Option<String>,

    /// Text to type for text type
    #[serde(default)]
    pub text: Option<String>,

    /// Modifiers for combo type: ctrl, alt, shift, cmd
    #[serde(default)]
    pub modifiers: Option<Vec<String>>,

    /// Keys for combo type
    #[serde(default)]
    pub keys: Option<Vec<String>>,

    /// Action: press, release, type
    #[serde(default)]
    pub action: Option<KeyboardAction>,
}

/// Arguments for mouse input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MouseInputArgs {
    /// Mouse action: move, click, double_click, right_click, scroll, drag
    pub action: MouseAction,

    /// X coordinate for move/click/drag
    #[serde(default)]
    pub x: Option<i32>,

    /// Y coordinate for move/click/drag
    #[serde(default)]
    pub y: Option<i32>,

    /// Mouse button for click actions
    #[serde(default)]
    pub button: Option<MouseButtonType>,

    /// Horizontal scroll delta
    #[serde(default)]
    pub delta_x: Option<i32>,

    /// Vertical scroll delta
    #[serde(default)]
    pub delta_y: Option<i32>,
}

/// Execute keyboard input
pub fn keyboard_input<P: Platform>(
    platform: &P,
    args: &KeyboardInputArgs,
) -> Result<CallToolResult, PcControllerError> {
    let input_type = args.input_type.clone().unwrap_or(InputType::Key);

    match input_type {
        InputType::Text => {
            let text = args.text.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("text required for text input".to_string()))?;
            platform.keyboard_type(text)?;
        }
        InputType::Key => {
            let key = args.key.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("key required for key input".to_string()))?;
            let action = match args.action.as_ref().unwrap_or(&KeyboardAction::Type) {
                KeyboardAction::Press => InputAction::Press,
                KeyboardAction::Release => InputAction::Release,
                KeyboardAction::Type => InputAction::Type,
            };
            platform.keyboard_key(key, action)?;
        }
        InputType::Combo => {
            let modifiers: Vec<KeyModifier> = args.modifiers.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("modifiers required for combo input".to_string()))?
                .iter()
                .filter_map(|m| match m.to_lowercase().as_str() {
                    "ctrl" | "control" => Some(KeyModifier::Ctrl),
                    "alt" | "option" => Some(KeyModifier::Alt),
                    "shift" => Some(KeyModifier::Shift),
                    "cmd" | "command" | "meta" => Some(KeyModifier::Cmd),
                    _ => None,
                })
                .collect();

            let keys: Vec<&str> = args.keys.as_ref()
                .ok_or_else(|| PcControllerError::InvalidArguments("keys required for combo input".to_string()))?
                .iter()
                .map(|s| s.as_str())
                .collect();

            platform.keyboard_combo(&keys, &modifiers)?;
        }
    }

    Ok(CallToolResult::success(vec![Content::text("Input sent successfully".to_string())]))
}

/// Execute mouse input
pub fn mouse_input<P: Platform>(
    platform: &P,
    args: &MouseInputArgs,
) -> Result<CallToolResult, PcControllerError> {
    match args.action {
        MouseAction::Move => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            platform.mouse_move(x, y)?;
        }
        MouseAction::Click | MouseAction::DoubleClick => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            platform.mouse_move(x, y)?;

            let button = match args.button.as_ref().unwrap_or(&MouseButtonType::Left) {
                MouseButtonType::Left => MouseButton::Left,
                MouseButtonType::Right => MouseButton::Right,
                MouseButtonType::Middle => MouseButton::Middle,
            };

            if args.action == MouseAction::DoubleClick {
                platform.mouse_click(button, InputAction::Type)?;
                platform.mouse_click(button, InputAction::Type)?;
            } else {
                platform.mouse_click(button, InputAction::Press)?;
                platform.mouse_click(button, InputAction::Release)?;
            }
        }
        MouseAction::RightClick => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            platform.mouse_move(x, y)?;
            platform.mouse_click(MouseButton::Right, InputAction::Press)?;
            platform.mouse_click(MouseButton::Right, InputAction::Release)?;
        }
        MouseAction::Scroll => {
            let delta_x = args.delta_x.unwrap_or(0);
            let delta_y = args.delta_y.unwrap_or(0);
            platform.mouse_scroll(delta_x, delta_y)?;
        }
        MouseAction::Drag => {
            let x = args.x.unwrap_or(0);
            let y = args.y.unwrap_or(0);
            platform.mouse_move(x, y)?;
            platform.mouse_click(MouseButton::Left, InputAction::Press)?;

            let delta_x = args.delta_x.unwrap_or(0);
            let delta_y = args.delta_y.unwrap_or(0);
            if delta_x != 0 || delta_y != 0 {
                platform.mouse_move(x + delta_x, y + delta_y)?;
            }

            platform.mouse_click(MouseButton::Left, InputAction::Release)?;
        }
    }

    Ok(CallToolResult::success(vec![Content::text("Mouse input sent successfully".to_string())]))
}
