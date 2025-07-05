# MCPadawan Setup for Claude Code

This guide explains how to configure MCPadawan to work with Claude Code using the built-in MCP management system.

## Quick Setup

Claude Code has a built-in MCP server management system accessed via `claude mcp` commands.

### Method 1: Add MCPadawan MCP Server (Recommended)

Add MCPadawan as an MCP server using the `claude mcp add` command:

```bash
# Add MCPadawan MCP server from the project directory
cd /Users/scott/development/projects/mcpadawan
claude mcp add mcpadawan cargo run -- --mcp
```

Or with full path:

```bash
# Add with full path specification
claude mcp add mcpadawan cargo "run" "--manifest-path" "/Users/scott/development/projects/mcpadawan/Cargo.toml" "--" "--mcp"
```

### Method 2: Using Built Binary (Faster)

First build the release binary with the latest changes, then add it:

```bash
# Build release binary (make sure to rebuild to include --mcp flag)
cd /Users/scott/development/projects/mcpadawan
cargo build --release

# Verify the --mcp flag is available
./target/release/mcpadawan --help

# Add the binary as MCP server
claude mcp add mcpadawan /Users/scott/development/projects/mcpadawan/target/release/mcpadawan -- --mcp
```

### Method 3: JSON Configuration

For more complex setups, use JSON configuration:

```bash
claude mcp add-json mcpadawan '{
  "command": "cargo",
  "args": ["run", "--manifest-path", "/Users/scott/development/projects/mcpadawan/Cargo.toml", "--", "--mcp"],
  "cwd": "/Users/scott/development/projects/mcpadawan"
}'
```

## Managing MCP Servers

### View Configured Servers
```bash
# List all configured MCP servers
claude mcp list

# Get details about MCPadawan server
claude mcp get mcpadawan
```

### Remove Server
```bash
# Remove MCPadawan MCP server
claude mcp remove mcpadawan
```

### Start MCP Server Mode
```bash
# Start Claude Code in MCP server mode (if needed for debugging)
claude mcp serve
```

## Usage with Claude Code

Once configured, you can use MCPadawan tools directly in your Claude Code session:

### Screenshot and Analysis
```
Take a screenshot and tell me what applications are currently open
```

### Mouse Control
```
Click on the center of the screen
```

### Keyboard Input
```
Type "Hello from Claude Code!" in the current window
```

### Application Management
```
Launch notepad and then type some text in it
```

### Gamepad Control (if ViGEm is installed)
```
Press the A button on the virtual gamepad
```

## Testing the Setup

1. **Add MCPadawan server**:
   ```bash
   cd /Users/scott/development/projects/mcpadawan
   claude mcp add mcpadawan cargo run -- --mcp
   ```

2. **Verify it's added**:
   ```bash
   claude mcp list
   ```

3. **Start a Claude Code session**:
   ```bash
   claude-code
   ```

4. **Test screenshot functionality**:
   ```
   Can you take a screenshot?
   ```

5. **Test mouse control**:
   ```
   Move the mouse to coordinates 500, 300
   ```

6. **Verify all tools are available**:
   ```
   What MCP tools do you have available?
   ```

## Development Workflow

For development, you can run both the HTTP server and MCP server simultaneously:

```bash
# Terminal 1: Run HTTP server for testing
cargo run

# Terminal 2: Update MCP server for development
claude mcp remove mcpadawan
claude mcp add mcpadawan cargo run -- --mcp

# Terminal 3: Start Claude Code
claude-code
```

This allows you to:
- Test HTTP endpoints with curl/browser
- Use MCP tools through Claude Code
- Debug both interfaces simultaneously
- Quickly update the MCP server during development

## Available MCP Tools

Once connected, Claude Code will have access to these MCPadawan tools:

| Tool | Description | Example Usage |
|------|-------------|---------------|
| `take_screenshot` | Capture screen with optional resizing | "Take a 50% scaled screenshot" |
| `mouse_click` | Click at coordinates | "Click at 300, 200 with right button" |
| `mouse_move` | Move mouse cursor | "Move mouse to top-left corner" |
| `mouse_scroll` | Scroll mouse wheel | "Scroll down 3 units" |
| `type_text` | Type keyboard input | "Type 'Hello World'" |
| `gamepad_input` | Control virtual gamepad | "Press A and B buttons simultaneously" |
| `start_application` | Launch programs | "Start calculator app" |
| `stop_application` | Stop running app | "Stop the current application" |

## Advanced Configuration

### Custom Build Configuration

If you want to use specific build features:

```bash
# With ViGEm support
claude mcp add mcpadawan cargo run --features vigem -- --mcp

# Release build for better performance
claude mcp add mcpadawan cargo run --release -- --mcp
```

### Multiple MCP Servers

You can configure multiple MCP servers:

```bash
# Add multiple servers
claude mcp add mcpadawan cargo run -- --mcp
claude mcp add other-server other-command -- --args

# List all configured servers
claude mcp list
```

## Troubleshooting

### Server Not Starting
```bash
# Test MCPadawan MCP mode directly
cd /Users/scott/development/projects/mcpadawan
cargo run -- --mcp
# Should start and wait for JSON-RPC input

# If using built binary, make sure it's up to date:
cargo build --release
./target/release/mcpadawan --help  # Should show --mcp flag
./target/release/mcpadawan --mcp   # Should start MCP mode
```

### Tools Not Available
```
List all available MCP tools
```
Claude Code should show MCPadawan tools if properly connected.

### Permission Issues
- **macOS**: May need to grant accessibility permissions
- **Windows**: May need to run as administrator for some operations
- **Screenshots**: May require screen recording permissions

### Performance
For better performance in production use:
```bash
# Build release binary first (ensure latest changes are included)
cd /Users/scott/development/projects/mcpadawan
cargo build --release

# Test the binary has --mcp flag
./target/release/mcpadawan --help

# Remove existing server and add binary version
claude mcp remove mcpadawan
claude mcp add mcpadawan /Users/scott/development/projects/mcpadawan/target/release/mcpadawan -- --mcp
```

## Integration Examples

### Automated UI Testing
```
Take a screenshot, find the login button, click it, and type my credentials
```

### System Automation
```
Open a text editor, type a quick note, and save it to the desktop
```

### Gaming Integration
```
Press the arrow keys to move the character in the game
```

### Accessibility Assistance
```
Help me navigate this application by taking screenshots and clicking on elements I describe
```

The MCP integration with Claude Code provides a seamless way to give Claude direct control over your system while maintaining the familiar conversational interface.