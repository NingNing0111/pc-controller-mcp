//! PC Controller MCP Server implementation

use crate::platform::Platform;
use crate::tools::input::{grid_mouse_input, keyboard_input, mouse_input, GridMouseInputArgs, KeyboardInputArgs, MouseInputArgs};
use crate::tools::screen::{capture_screen, CaptureScreenArgs};
use crate::tools::window::{focus_window, list_windows, FocusWindowArgs};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{
    tool, tool_handler, tool_router,
    RoleServer, ServerHandler, service::RequestContext,
};
use std::sync::Arc;

/// PC Controller - implements MCP ServerHandler with all PC control tools
pub struct PcController<P: Platform + 'static> {
    platform: Arc<P>,
    tool_router: ToolRouter<Self>,
}

impl<P: Platform + 'static> PcController<P> {
    pub fn new(platform: P) -> Self {
        Self {
            platform: Arc::new(platform),
            tool_router: Self::tool_router(),
        }
    }
}

impl<P: Platform + 'static> Clone for PcController<P> {
    fn clone(&self) -> Self {
        Self {
            platform: self.platform.clone(),
            tool_router: self.tool_router.clone(),
        }
    }
}

#[tool_router]
impl<P: Platform + 'static> PcController<P> {
    #[tool(name = "capture_screen", description = "Capture screen as image. Modes: fullscreen, window, region.")]
    fn capture_screen(
        &self,
        Parameters(args): Parameters<CaptureScreenArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        capture_screen(self.platform.as_ref(), &args)
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }

    #[tool(name = "list_windows", description = "List all visible windows with their information.")]
    fn list_windows(
        &self,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        list_windows(self.platform.as_ref())
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }

    #[tool(name = "focus_window", description = "Focus (bring to front) a specific window by its ID.")]
    fn focus_window(
        &self,
        Parameters(args): Parameters<FocusWindowArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        focus_window(self.platform.as_ref(), &args)
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }

    #[tool(name = "keyboard_input", description = "Send keyboard input: type text, press keys, or key combos with modifiers.")]
    fn keyboard_input(
        &self,
        Parameters(args): Parameters<KeyboardInputArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        keyboard_input(self.platform.as_ref(), &args)
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }

    #[tool(name = "mouse_input", description = "Send mouse input: move, click, double_click, right_click, scroll, drag.")]
    fn mouse_input(
        &self,
        Parameters(args): Parameters<MouseInputArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        mouse_input(self.platform.as_ref(), &args)
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }

    #[tool(name = "grid_mouse_input", description = "Send mouse input using grid cell ID (like B3) with optional offset. Grid IDs match capture_screen grid overlay labels.")]
    fn grid_mouse_input(
        &self,
        Parameters(args): Parameters<GridMouseInputArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        grid_mouse_input(self.platform.as_ref(), &args)
            .map_err(|e| rmcp::ErrorData {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: e.to_string().into(),
                data: None,
            })
    }
}

#[tool_handler]
impl<P: Platform + 'static> ServerHandler for PcController<P> {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_protocol_version(ProtocolVersion::V_2024_11_05)
        .with_instructions(
            "PC Controller MCP Server - Provides screen capture, window management, and input simulation tools.\n\n\
            Tools:\n\
            - capture_screen: Capture screen (fullscreen/window/region)\n\
            - list_windows: List all visible windows\n\
            - focus_window: Bring a window to front by its ID\n\
            - keyboard_input: Send keyboard input\n\
            - mouse_input: Send mouse input".to_string(),
        )
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, rmcp::ErrorData> {
        tracing::info!("PC Controller MCP initializing...");
        Ok(self.get_info())
    }
}
