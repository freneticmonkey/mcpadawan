# MCPadawan API Documentation

MCPadawan provides a RESTful HTTP API for controlling mouse, keyboard, gamepad, and screenshot functionality.

## Base URL

```
http://127.0.0.1:5050
```

## Endpoints

### Application Management

#### Start Application
**POST** `/start_app`

Launches an external application.

**Request Body:**
```json
{
  "exe_path": "string",    // Required: Path to executable
  "args": ["string"]       // Optional: Command line arguments
}
```

**Response:**
- `200 OK`: Returns "Application started" or error message

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/start_app \
  -H "Content-Type: application/json" \
  -d '{"exe_path": "C:/Windows/System32/notepad.exe"}'
```

#### Stop Application
**POST** `/stop_app`

Terminates the currently running application.

**Response:**
- `200 OK`: Returns "Application stopped", "No application running", or error message

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/stop_app
```

### Mouse Control

#### Mouse Click
**POST** `/click`

Performs a mouse click at specified coordinates.

**Request Body:**
```json
{
  "x": 100,               // Required: X coordinate
  "y": 200,               // Required: Y coordinate
  "button": "left"        // Optional: "left", "right", "middle" (default: "left")
}
```

**Response:**
- `200 OK`: Returns "Clicked"

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/click \
  -H "Content-Type: application/json" \
  -d '{"x": 300, "y": 200, "button": "right"}'
```

#### Mouse Move
**POST** `/mousemove`

Moves the mouse cursor.

**Request Body:**
```json
{
  "x": 100,               // Required: X coordinate
  "y": 200,               // Required: Y coordinate
  "relative": false       // Optional: true for relative movement (default: false)
}
```

**Response:**
- `200 OK`: Returns "Moved"

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/mousemove \
  -H "Content-Type: application/json" \
  -d '{"x": 500, "y": 300, "relative": false}'
```

#### Mouse Scroll
**POST** `/scroll`

Scrolls the mouse wheel.

**Request Body:**
```json
{
  "x": 0,                 // Required: Horizontal scroll amount
  "y": 3                  // Required: Vertical scroll amount
}
```

**Response:**
- `200 OK`: Returns "Scrolled"

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/scroll \
  -H "Content-Type: application/json" \
  -d '{"x": 0, "y": -3}'
```

### Keyboard Control

#### Type Text
**POST** `/type`

Types the specified text.

**Request Body:**
```json
{
  "text": "Hello, world!" // Required: Text to type
}
```

**Response:**
- `200 OK`: Returns "Typed"

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/type \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello from MCPadawan!"}'
```

### Screenshot

#### Capture Screenshot
**GET** `/screenshot`

Captures a screenshot of the primary display with optional resizing.

**Query Parameters:**
- `scale` (optional): Scaling factor (e.g., 0.5 for 50% size)
- `width` (optional): Target width in pixels
- `height` (optional): Target height in pixels

**Parameter Combinations:**
- No parameters: Original resolution
- `scale` only: Proportional scaling
- `width` only: Fixed width, aspect ratio preserved
- `height` only: Fixed height, aspect ratio preserved
- `width` + `height`: Exact dimensions

**Response:**
- `200 OK`: PNG image data
- `Content-Type: image/png`

**Examples:**
```bash
# Original size
curl http://127.0.0.1:5050/screenshot -o screenshot.png

# 50% scale
curl "http://127.0.0.1:5050/screenshot?scale=0.5" -o small.png

# Fixed width, maintain aspect ratio
curl "http://127.0.0.1:5050/screenshot?width=800" -o width800.png

# Fixed height, maintain aspect ratio
curl "http://127.0.0.1:5050/screenshot?height=600" -o height600.png

# Exact dimensions
curl "http://127.0.0.1:5050/screenshot?width=1920&height=1080" -o exact.png
```

### Gamepad Control

#### Send Gamepad Input
**POST** `/gamepad`

Sends input to the virtual Xbox 360 controller (requires ViGEm).

**Request Body:**
```json
{
  "button_a": true,       // Required: A button state
  "button_b": false,      // Required: B button state
  "left_trigger": 255,    // Required: Left trigger (0-255)
  "right_trigger": 0,     // Required: Right trigger (0-255)
  "left_thumb_x": 32767,  // Required: Left stick X (-32768 to 32767)
  "left_thumb_y": 0,      // Required: Left stick Y (-32768 to 32767)
  "right_thumb_x": 0,     // Required: Right stick X (-32768 to 32767)
  "right_thumb_y": 0      // Required: Right stick Y (-32768 to 32767)
}
```

**Response:**
- `200 OK`: Returns "Gamepad input sent", "No gamepad connected", or error message

**Example:**
```bash
curl -X POST http://127.0.0.1:5050/gamepad \
  -H "Content-Type: application/json" \
  -d '{
    "button_a": true,
    "button_b": false,
    "left_trigger": 128,
    "right_trigger": 0,
    "left_thumb_x": 16000,
    "left_thumb_y": 0,
    "right_thumb_x": 0,
    "right_thumb_y": 0
  }'
```

## Error Responses

All endpoints may return the following error responses:

- `400 Bad Request`: Invalid JSON or missing required fields
- `500 Internal Server Error`: Server error (e.g., screenshot capture failure)

Error responses include descriptive messages in the response body.

## Rate Limiting

There are no built-in rate limits, but excessive requests may impact system performance.

## Security Considerations

- MCPadawan runs on localhost by default (127.0.0.1)
- No authentication is required
- Use caution when exposing the server to network access
- The server can execute arbitrary applications via `/start_app`

## Testing

The API includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Test specific functionality
cargo test screenshot
cargo test mouse
cargo test gamepad
```

## Version

API Version: 0.1.0  
Compatible with MCPadawan v0.1.0+