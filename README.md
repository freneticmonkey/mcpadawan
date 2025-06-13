# 🎮 MCPadawan

A Rust-based **Mouse, Keyboard, Screenshot, and Virtual Gamepad Control Server** for game developers, control tinkerers, and Jedi Masters alike.

MCPadawan gives you **force-like powers** over input simulation and gamepad emulation, with a RESTful API and native ViGEm integration — perfect for automation, prototyping, or powering your own input AI.

---

## ✨ Features

- 🖱 Mouse input: absolute & relative movement, clicks, scrolling
- ⌨ Keyboard typing and sequences
- 🧠 Launch and stop external applications
- 🖼 Screenshot the primary display
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
```

---

## 👨‍🎓 Why “MCPadawan”?

MCPadawan is your input-sidekick: a young, programmable apprentice in the **Machine Control Protocol** arts. But with a little training, it can move your mouse, wield your keyboard, and pilot your gamepad like a true master.

---

## 📝 License

MIT License.  
Built with 💛 + 🦀 + 🎮 in Rust.