# PC Controller MCP Server

A Model Context Protocol (MCP) server for PC control that enables AI agents to interact with your computer through screen capture, window management, and input simulation.

## Features

- **Screen Capture** - Full screen, window-specific, or region-based screenshots
- **Window Management** - List all visible windows and focus specific windows
- **Input Simulation** - Control keyboard and mouse programmatically
- **Multi-Protocol Support** - STDIO, HTTP, and WebSocket transports
- **Cross-Platform** - macOS primary support, Windows planned

## Installation

### Prerequisites

- Rust 1.75+
- macOS or Windows

### Build from Source

```bash
git clone https://github.com/NingNing0111/pc-controller-mcp.git
cd pc-controller-mcp
cargo build --release
```

The binary will be at `target/release/pc-controller-mcp`.

## Usage

### STDIO Protocol (Recommended for Claude Code)

```bash
pc-controller-mcp --protocol stdio
```

### HTTP Protocol

```bash
pc-controller-mcp --protocol http --addr 127.0.0.1:8080
```

With CORS enabled:

```bash
pc-controller-mcp --protocol http --addr 127.0.0.1:8080 --cors
```

### WebSocket Protocol

```bash
pc-controller-mcp --protocol ws --addr 127.0.0.1:8080
```

## MCP Tools

### `capture_screen`

Capture screen as image.

```json
{
  "mode": "fullscreen",  // "fullscreen" | "window" | "region"
  "window_id": "12345",  // required for window mode
  "region": [x, y, width, height],  // required for region mode
  "display_id": 0  // optional, for multi-monitor
}
```

### `list_windows`

List all visible windows with their information.

```json
{}
```

Returns:

```json
[
  {
    "id": "12345",
    "title": "Safari",
    "app_name": "Safari",
    "bounds": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
    "is_minimized": false
  }
]
```

### `focus_window`

Bring a specific window to the foreground.

```json
{
  "window_id": "12345"
}
```

### `keyboard_input`

Send keyboard input.

```json
{
  "input_type": "text", // "key" | "text" | "combo"
  "text": "Hello, World!", // for text type
  "key": "a", // for key type
  "modifiers": ["ctrl", "shift"], // for combo type
  "keys": ["a"], // for combo type
  "action": "type" // "press" | "release" | "type"
}
```

### `mouse_input`

Send mouse input.

```json
{
  "action": "click", // "move" | "click" | "double_click" | "right_click" | "scroll" | "drag"
  "x": 100,
  "y": 200,
  "button": "left", // "left" | "right" | "middle"
  "delta_x": 0,
  "delta_y": -10
}
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 MCP Client/Agent                 │
└─────────────────────┬───────────────────────────┘
                      │ MCP Protocol
┌─────────────────────▼───────────────────────────┐
│              PC Controller Server                │
│  ┌───────────────────────────────────────────┐  │
│  │            Protocol Layer                  │  │
│  │   STDIO  │  HTTP Streamable  │  WebSocket │  │
│  └─────────────────────┬─────────────────────┘  │
│  ┌─────────────────────▼─────────────────────┐  │
│  │           Tool Router & Handler            │  │
│  │  capture_screen │ list_windows │ input    │  │
│  └─────────────────────┬─────────────────────┘  │
│  ┌─────────────────────▼─────────────────────┐  │
│  │          Platform Abstraction             │  │
│  └─────────────────────┬─────────────────────┘  │
└────────────────────────┼────────────────────────┘
                         │
        ┌────────────────┴────────────────┐
        │         Platform Layer          │
        │   macOS Platform  │  Windows   │
        │   (xcap/enigo)    │  Platform  │
        └─────────────────────────────────┘
```

## Platform Capabilities

| Feature        | macOS | Windows |
| -------------- | ----- | ------- |
| Screen Capture | ✅    | ✅      |
| Window List    | ✅    | ✅      |
| Window Focus   | ✅    | ✅      |
| Keyboard Input | ✅    | ✅      |
| Mouse Input    | ✅    | ✅      |

## Security Notes

This tool requires elevated privileges for input simulation and window management. On macOS, you may need to grant **Accessibility permissions** in System Settings > Privacy & Security > Accessibility.

## License

MIT
