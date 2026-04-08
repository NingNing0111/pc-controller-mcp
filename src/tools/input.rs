//! Input simulation tools

use crate::error::PcControllerError;
use crate::platform::{InputAction, KeyModifier, MouseButton, Platform};
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

/// Arguments for grid-based mouse input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GridMouseInputArgs {
    /// Mouse action: move, click, double_click, right_click, drag
    pub action: MouseAction,

    /// Grid cell ID like "B3" (column letter + row number)
    pub grid_id: String,

    /// X offset from grid cell center in pixels (optional)
    #[serde(default)]
    pub offset_x: Option<i32>,

    /// Y offset from grid cell center in pixels (optional)
    #[serde(default)]
    pub offset_y: Option<i32>,

    /// Mouse button for click actions (default: left)
    #[serde(default)]
    pub button: Option<MouseButtonType>,

    /// Horizontal delta for drag operations
    #[serde(default)]
    pub delta_x: Option<i32>,

    /// Vertical delta for drag operations
    #[serde(default)]
    pub delta_y: Option<i32>,

    /// Grid columns (default: 12, should match capture_screen grid_cols)
    #[serde(default = "default_grid_cols")]
    pub grid_cols: u32,

    /// Grid rows (default: 8, should match capture_screen grid_rows)
    #[serde(default = "default_grid_rows")]
    pub grid_rows: u32,
}

fn default_grid_cols() -> u32 { 12 }
fn default_grid_rows() -> u32 { 8 }

/// Parse grid ID like "B3" into (col, row) 0-indexed coordinates
/// Returns (col, row) where col is 0-indexed (A=0, B=1, ...) and row is 0-indexed (1→0, 2→1, ...)
pub fn parse_grid_id(grid_id: &str) -> Result<(u32, u32), String> {
    let grid_id = grid_id.trim().to_uppercase();
    if grid_id.is_empty() {
        return Err("Grid ID cannot be empty".to_string());
    }

    let mut chars = grid_id.chars();
    let col_char = chars.next().ok_or("Grid ID must have a column letter")?;
    let rest: String = chars.collect();

    if !col_char.is_ascii_alphabetic() {
        return Err(format!("Invalid column letter '{}' in grid ID '{}'", col_char, grid_id));
    }

    let col = (col_char as u32) - (b'A' as u32);

    if rest.is_empty() {
        return Err(format!("Grid ID '{}' missing row number", grid_id));
    }

    let row: u32 = rest.parse().map_err(|_| format!("Invalid row number '{}' in grid ID '{}'", rest, grid_id))?;

    if row == 0 {
        return Err(format!("Row in grid ID '{}' must be 1-based (1-{})", grid_id, u32::MAX));
    }

    // Convert to 0-indexed
    Ok((col, row - 1))
}

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

impl std::fmt::Display for MouseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseAction::Move => write!(f, "move"),
            MouseAction::Click => write!(f, "click"),
            MouseAction::DoubleClick => write!(f, "double_click"),
            MouseAction::RightClick => write!(f, "right_click"),
            MouseAction::Scroll => write!(f, "scroll"),
            MouseAction::Drag => write!(f, "drag"),
        }
    }
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

/// Execute grid-based mouse input
pub fn grid_mouse_input<P: Platform>(
    platform: &P,
    args: &GridMouseInputArgs,
) -> Result<CallToolResult, PcControllerError> {
    // Get screen dimensions for grid calculation
    let dims = platform.get_screen_dimensions();

    // Parse grid ID
    let (col, row) = parse_grid_id(&args.grid_id)
        .map_err(|e| PcControllerError::InvalidArguments(e))?;

    // Validate grid dimensions
    if args.grid_cols == 0 {
        return Err(PcControllerError::InvalidArguments("grid_cols must be positive".to_string()));
    }
    if args.grid_rows == 0 {
        return Err(PcControllerError::InvalidArguments("grid_rows must be positive".to_string()));
    }

    // Calculate cell dimensions
    let cell_width = dims.width as f64 / args.grid_cols as f64;
    let cell_height = dims.height as f64 / args.grid_rows as f64;

    // Calculate grid center position
    let center_x = (col as f64 * cell_width + cell_width / 2.0) as i32;
    let center_y = (row as f64 * cell_height + cell_height / 2.0) as i32;

    // Apply offset if provided
    let offset_x = args.offset_x.unwrap_or(0);
    let offset_y = args.offset_y.unwrap_or(0);
    let final_x = center_x + offset_x;
    let final_y = center_y + offset_y;

    // Execute the action with calculated coordinates
    match args.action {
        MouseAction::Move => {
            platform.mouse_move(final_x, final_y)?;
        }
        MouseAction::Click | MouseAction::DoubleClick => {
            platform.mouse_move(final_x, final_y)?;

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
            platform.mouse_move(final_x, final_y)?;
            platform.mouse_click(MouseButton::Right, InputAction::Press)?;
            platform.mouse_click(MouseButton::Right, InputAction::Release)?;
        }
        MouseAction::Drag => {
            // Move to starting position
            platform.mouse_move(final_x, final_y)?;
            platform.mouse_click(MouseButton::Left, InputAction::Press)?;

            // Apply drag delta
            let delta_x = args.delta_x.unwrap_or(0);
            let delta_y = args.delta_y.unwrap_or(0);
            if delta_x != 0 || delta_y != 0 {
                platform.mouse_move(final_x + delta_x, final_y + delta_y)?;
            }

            platform.mouse_click(MouseButton::Left, InputAction::Release)?;
        }
        MouseAction::Scroll => {
            // Grid scroll uses offset as scroll delta
            let delta_x = args.offset_x.unwrap_or(0);
            let delta_y = args.offset_y.unwrap_or(0);
            platform.mouse_scroll(delta_x, delta_y)?;
        }
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Grid mouse input sent: {} at {} (offset: {}, {})",
        args.action.to_string(),
        args.grid_id,
        offset_x,
        offset_y
    ))]))
}
