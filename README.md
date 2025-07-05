# 🎮 MCPadawan

A Rust-based **Mouse, Keyboard, Screenshot, and Virtual Gamepad Control Server** for game developers, control tinkerers, and Jedi Masters alike.

MCPadawan gives you **force-like powers** over input simulation and gamepad emulation, with a RESTful API and native ViGEm integration — perfect for automation, prototyping, or powering your own input AI.

---

## ✨ Features

- 🖱 Mouse input: absolute & relative movement, clicks, scrolling
- ⌨ Keyboard typing and sequences
- 🧠 Launch and stop external applications
- 🖼 Screenshot the primary display with configurable resolution and scaling
- 🎮 Xbox 360 virtual controller emulation via ViGEm
- 🔌 HTTP API using `axum` (async + fast)

---

## 🛠 Requirements

- 🦀 Rust (Edition 2021+): [Install via rustup](https://rustup.rs)
- 🪟 Windows 10 or 11
- ✅ [ViGEmBus driver](https://vigem.org) installed

---

## 🚀 Getting Started

### 1. Install the ViGEmBus Driver

[🔗 https://vigem.org/projects/ViGEm/ViGEmBus/](https://vigem.org/projects/ViGEm/ViGEmBus/)

### 2. Clone & Run the Server

```bash
git clone https://github.com/yourname/mcpadawan.git
cd mcpadawan

# Run with Makefile
make run

# Or directly with Cargo
cargo run
```

Server will start at: [http://127.0.0.1:5050](http://127.0.0.1:5050)

---

## 📡 Example API Usage

### ▶️ Launch an Application

```http
POST /start_app
{
  "exe_path": "C:/Windows/System32/notepad.exe"
}
```

### 🖱 Mouse Click

```http
POST /click
{
  "x": 300,
  "y": 200,
  "button": "left"
}
```

### ⌨ Type Text

```http
POST /type
{
  "text": "Hello, world!"
}
```

### 🖼 Take Screenshot

```http
# Basic screenshot
GET /screenshot

# Scaled screenshot (50% of original size)
GET /screenshot?scale=0.5

# Fixed width, maintaining aspect ratio
GET /screenshot?width=800

# Fixed height, maintaining aspect ratio
GET /screenshot?height=600

# Exact dimensions
GET /screenshot?width=1920&height=1080
```

### 🎮 Send Gamepad Input

```http
POST /gamepad
{
  "button_a": true,
  "button_b": false,
  "left_trigger": 128,
  "right_trigger": 0,
  "left_thumb_x": 1000,
  "left_thumb_y": 0,
  "right_thumb_x": 0,
  "right_thumb_y": 0
}
```

---

## 🧪 Development Commands

```bash
make fmt         # Format code
make check       # Syntax/type checks
make build       # Build release binary
make clean       # Clean target dir
make driver-check  # Check ViGEm driver (Windows only)

# Testing
cargo test                    # Run all tests
cargo test --test integration_tests  # Run integration tests only
cargo test screenshot         # Run screenshot-related tests

# MCP Mode
cargo run -- --mcp          # Run as MCP server for Claude integration
```

---

## 🤖 Claude Integration

MCPadawan can integrate directly with Claude Desktop and Claude Code as an MCP (Model Context Protocol) server:

### Claude Code (CLI)
```bash
# Add MCPadawan as an MCP server
cd /path/to/mcpadawan
claude mcp add mcpadawan cargo run -- --mcp

# Then start Claude Code normally
claude
```

### Claude Desktop
Add to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "mcpadawan": {
      "command": "cargo",
      "args": ["run", "--", "--mcp"],
      "cwd": "/path/to/mcpadawan"
    }
  }
}
```

📖 **Setup Guides:**
- [Claude Code Setup](CLAUDE_CODE_SETUP.md) - For CLI integration
- [Claude Desktop Setup](MCP_SETUP.md) - For desktop app integration

---

## 👨‍🎓 Why “MCPadawan”?

MCPadawan is your input-sidekick: a young, programmable apprentice in the **Machine Control Protocol** arts. But with a little training, it can move your mouse, wield your keyboard, and pilot your gamepad like a true master.

---

## 📝 License

MIT License.  
Built with 💛 + 🦀 + 🎮 in Rust.
