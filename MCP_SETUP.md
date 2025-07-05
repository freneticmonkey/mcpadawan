# MCPadawan MCP Server Setup

This guide explains how to configure MCPadawan to work with Claude Desktop as an MCP (Model Context Protocol) server.

## Overview

MCPadawan can run in two modes:
1. **HTTP Server Mode** (default): Traditional REST API server
2. **MCP Server Mode**: Integrates directly with Claude Desktop

## Prerequisites

1. **Rust installed**: [Get Rust](https://rustup.rs/)
2. **Claude Desktop**: [Download Claude Desktop](https://claude.ai/download)
3. **Windows 10/11** (for ViGEm gamepad support)
4. **ViGEmBus driver** (optional, for gamepad features): [Download ViGEmBus](https://vigem.org/)

## Setup Instructions

### 1. Build MCPadawan

```bash
cd /path/to/mcpadawan
cargo build --release
```

### 2. Configure Claude Desktop

Claude Desktop looks for MCP server configurations in:

**macOS:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Windows:**
```
%APPDATA%\Claude\claude_desktop_config.json
```

**Linux:**
```
~/.config/Claude/claude_desktop_config.json
```

### 3. Add MCPadawan Configuration

Edit the `claude_desktop_config.json` file and add the MCPadawan server:

```json
{
  "mcpServers": {
    "mcpadawan": {
      "command": "cargo",
      "args": ["run", "--", "--mcp"],
      "cwd": "/path/to/your/mcpadawan"
    }
  }
}
```

**Important:** Update the `cwd` path to match your actual MCPadawan installation directory.

**Example for this setup:**
```json
{
  "mcpServers": {
    "mcpadawan": {
      "command": "cargo",
      "args": ["run", "--", "--mcp"],
      "cwd": "/Users/scott/development/projects/mcpadawan"
    }
  }
}
```

### 4. Alternative: Use Built Binary

If you prefer to use the built binary instead of `cargo run`:

```json
{
  "mcpServers": {
    "mcpadawan": {
      "command": "/path/to/mcpadawan/target/release/mcpadawan",
      "args": ["--mcp"]
    }
  }
}
```

### 5. Restart Claude Desktop

After updating the configuration file, restart Claude Desktop for the changes to take effect.

## Available MCP Tools

Once configured, Claude will have access to these tools:

### 🖼️ **take_screenshot**
Capture screenshots with optional resizing
- `scale`: Scaling factor (0.1 - 2.0)
- `width`: Target width in pixels
- `height`: Target height in pixels

### 🖱️ **mouse_click**
Click at specified coordinates
- `x`, `y`: Click coordinates
- `button`: "left", "right", or "middle"

### 🖱️ **mouse_move**
Move mouse cursor
- `x`, `y`: Target coordinates
- `relative`: True for relative movement

### 🖱️ **mouse_scroll**
Scroll mouse wheel
- `x`: Horizontal scroll amount
- `y`: Vertical scroll amount

### ⌨️ **type_text**
Type text using keyboard
- `text`: Text to type

### 🎮 **gamepad_input**
Control virtual Xbox 360 controller (requires ViGEm)
- `button_a`, `button_b`: Button states
- `left_trigger`, `right_trigger`: Trigger values (0-255)
- `left_thumb_x`, `left_thumb_y`: Left stick (-32768 to 32767)
- `right_thumb_x`, `right_thumb_y`: Right stick (-32768 to 32767)

### 🚀 **start_application**
Launch external applications
- `exe_path`: Path to executable
- `args`: Command line arguments (optional)

### ⏹️ **stop_application**
Stop currently running application
- No parameters required

## Testing the Setup

1. Start Claude Desktop
2. Look for MCPadawan in the available tools/integrations
3. Try asking Claude to take a screenshot: "Can you take a screenshot?"
4. Test mouse control: "Click at coordinates 100, 200"

## Troubleshooting

### MCPadawan Not Appearing in Claude

1. **Check the configuration file path** - Make sure you're editing the correct file
2. **Verify the cwd path** - Ensure the path points to your MCPadawan directory
3. **Check file permissions** - Make sure Claude can read the config file
4. **Restart Claude Desktop** - Configuration changes require a restart

### MCP Server Fails to Start

1. **Check Rust installation**: `rustc --version`
2. **Verify dependencies**: `cargo check`
3. **Check file paths** - Ensure all paths in the config are correct
4. **Run manually** - Test with `cargo run -- --mcp` from the project directory

### Tools Not Working

1. **Windows permissions** - Some operations may require administrator privileges
2. **ViGEm driver** - Gamepad features require ViGEm to be installed
3. **Screen permissions** - Screenshot functionality may need screen recording permissions

## Development Mode

For development, you can run MCPadawan in MCP mode directly:

```bash
# Run as MCP server
cargo run -- --mcp

# Run as HTTP server (default)
cargo run

# Run with ViGEm disabled
cargo run -- --mcp --disable-vigem
```

## Security Considerations

- MCPadawan provides powerful system control capabilities
- Only run it in trusted environments
- Be cautious when giving Claude access to application launching
- The tool can control your mouse, keyboard, and launch programs
- Consider running in a sandboxed environment for testing

## Example Usage with Claude

Once set up, you can ask Claude to:

- "Take a screenshot and analyze what's on my screen"
- "Click the button at coordinates 300, 400"
- "Type 'Hello World' for me"
- "Move the mouse to the center of the screen"
- "Launch notepad"
- "Press the A button on the gamepad"

The MCP integration allows Claude to seamlessly control your system while maintaining the conversational interface you're used to.