// MCP (Model Context Protocol) server implementation for MCPadawan
// Simple JSON-RPC based implementation

use crate::*;
use base64::Engine;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(serde::Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub struct McpServer {
    state: AppState,
}

impl McpServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await? {
                0 => break, // EOF
                _ => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    // eprintln!("Received: {}", trimmed);
                    
                    match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                        Ok(request) => {
                            let response = self.handle_request(request).await;
                            // Only send response if it has an id or result/error
                            if response.id.is_some() || response.result.is_some() || response.error.is_some() {
                                let response_json = serde_json::to_string(&response)?;
                                // eprintln!("Sending: {}", response_json);
                                stdout.write_all(response_json.as_bytes()).await?;
                                stdout.write_all(b"\n").await?;
                                stdout.flush().await?;
                            }
                        }
                        Err(e) => {
                            // eprintln!("JSON parse error: {}", e);
                            // Try to extract id from malformed JSON for error response
                            let error_response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: None,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32700,
                                    message: "Parse error".to_string(),
                                    data: Some(json!({"error": e.to_string()})),
                                }),
                            };
                            let error_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(error_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id).await,
            "initialized" => self.handle_initialized(request.id).await,
            "tools/list" => self.handle_list_tools(request.id).await,
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    async fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "mcpadawan",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }
    }

    async fn handle_initialized(&self, id: Option<Value>) -> JsonRpcResponse {
        // eprintln!("Received initialized notification");
        // Notifications don't require a response according to JSON-RPC spec
        // Only return a response if there's an id (making it a request, not a notification)
        if id.is_some() {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({})),
                error: None,
            }
        } else {
            // For notifications, return an empty response that won't be sent
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: None,
            }
        }
    }

    async fn handle_list_tools(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = json!({
            "tools": [
                {
                    "name": "take_screenshot",
                    "description": "Take a screenshot of the primary display with optional resizing",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "scale": {
                                "type": "number",
                                "description": "Scaling factor (e.g., 0.5 for 50% size reduction)",
                                "minimum": 0.1,
                                "maximum": 2.0
                            },
                            "width": {
                                "type": "integer",
                                "description": "Target width in pixels",
                                "minimum": 1
                            },
                            "height": {
                                "type": "integer",
                                "description": "Target height in pixels",
                                "minimum": 1
                            }
                        },
                        "additionalProperties": false
                    }
                },
                {
                    "name": "mouse_click",
                    "description": "Click the mouse at specified coordinates",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "x": {
                                "type": "integer",
                                "description": "X coordinate for the click"
                            },
                            "y": {
                                "type": "integer",
                                "description": "Y coordinate for the click"
                            },
                            "button": {
                                "type": "string",
                                "enum": ["left", "right", "middle"],
                                "description": "Mouse button to click",
                                "default": "left"
                            }
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "mouse_move",
                    "description": "Move the mouse cursor to specified coordinates",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "x": {
                                "type": "integer",
                                "description": "X coordinate to move to"
                            },
                            "y": {
                                "type": "integer",
                                "description": "Y coordinate to move to"
                            },
                            "relative": {
                                "type": "boolean",
                                "description": "Whether movement is relative to current position",
                                "default": false
                            }
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "mouse_scroll",
                    "description": "Scroll the mouse wheel",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "x": {
                                "type": "integer",
                                "description": "Horizontal scroll amount"
                            },
                            "y": {
                                "type": "integer",
                                "description": "Vertical scroll amount"
                            }
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "type_text",
                    "description": "Type text using the keyboard",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to type"
                            }
                        },
                        "required": ["text"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "gamepad_input",
                    "description": "Send input to virtual Xbox 360 controller (requires ViGEm)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "button_a": {
                                "type": "boolean",
                                "description": "A button state"
                            },
                            "button_b": {
                                "type": "boolean",
                                "description": "B button state"
                            },
                            "left_trigger": {
                                "type": "integer",
                                "description": "Left trigger value (0-255)",
                                "minimum": 0,
                                "maximum": 255
                            },
                            "right_trigger": {
                                "type": "integer",
                                "description": "Right trigger value (0-255)",
                                "minimum": 0,
                                "maximum": 255
                            },
                            "left_thumb_x": {
                                "type": "integer",
                                "description": "Left stick X axis (-32768 to 32767)",
                                "minimum": -32768,
                                "maximum": 32767
                            },
                            "left_thumb_y": {
                                "type": "integer",
                                "description": "Left stick Y axis (-32768 to 32767)",
                                "minimum": -32768,
                                "maximum": 32767
                            },
                            "right_thumb_x": {
                                "type": "integer",
                                "description": "Right stick X axis (-32768 to 32767)",
                                "minimum": -32768,
                                "maximum": 32767
                            },
                            "right_thumb_y": {
                                "type": "integer",
                                "description": "Right stick Y axis (-32768 to 32767)",
                                "minimum": -32768,
                                "maximum": 32767
                            }
                        },
                        "required": ["button_a", "button_b", "left_trigger", "right_trigger", "left_thumb_x", "left_thumb_y", "right_thumb_x", "right_thumb_y"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "start_application",
                    "description": "Launch an external application",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "exe_path": {
                                "type": "string",
                                "description": "Path to the executable to launch"
                            },
                            "args": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Command line arguments (optional)"
                            }
                        },
                        "required": ["exe_path"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "stop_application",
                    "description": "Stop the currently running application",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }
                }
            ]
        });

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools),
            error: None,
        }
    }

    async fn handle_call_tool(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match tool_name {
            "take_screenshot" => self.handle_screenshot_tool(arguments).await,
            "mouse_click" => self.handle_mouse_click_tool(arguments).await,
            "mouse_move" => self.handle_mouse_move_tool(arguments).await,
            "mouse_scroll" => self.handle_mouse_scroll_tool(arguments).await,
            "type_text" => self.handle_type_text_tool(arguments).await,
            "gamepad_input" => self.handle_gamepad_input_tool(arguments).await,
            "start_application" => self.handle_start_app_tool(arguments).await,
            "stop_application" => self.handle_stop_app_tool(arguments).await,
            _ => json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unknown tool: {}", tool_name)
                }],
                "isError": true
            }),
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    async fn handle_screenshot_tool(&self, arguments: Value) -> Value {
        let query = ScreenshotQuery {
            scale: arguments.get("scale").and_then(|v| v.as_f64().map(|f| f as f32)),
            width: arguments.get("width").and_then(|v| v.as_u64().map(|u| u as u32)),
            height: arguments.get("height").and_then(|v| v.as_u64().map(|u| u as u32)),
        };

        // Capture screenshot using same logic as HTTP endpoint
        let screens = match screenshots::Screen::all() {
            Ok(screens) => screens,
            Err(_) => {
                return json!({
                    "content": [{
                        "type": "text",
                        "text": "Failed to get screens"
                    }],
                    "isError": true
                });
            }
        };

        let screen = match screens.into_iter().next() {
            Some(screen) => screen,
            None => {
                return json!({
                    "content": [{
                        "type": "text",
                        "text": "No screen found"
                    }],
                    "isError": true
                });
            }
        };

        let image = match screen.capture() {
            Ok(image) => image,
            Err(_) => {
                return json!({
                    "content": [{
                        "type": "text",
                        "text": "Failed to capture screenshot"
                    }],
                    "isError": true
                });
            }
        };

        // Convert screenshot to RgbaImage
        let rgba_image = match image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
            image.width() as u32,
            image.height() as u32,
            image.buffer().to_vec(),
        ) {
            Some(img) => img,
            None => {
                return json!({
                    "content": [{
                        "type": "text",
                        "text": "Failed to convert image format"
                    }],
                    "isError": true
                });
            }
        };

        // Apply scaling or resize based on parameters
        let processed_image = if let Some(scale) = query.scale {
            let new_width = (rgba_image.width() as f32 * scale) as u32;
            let new_height = (rgba_image.height() as f32 * scale) as u32;
            image::imageops::resize(&rgba_image, new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else if let (Some(width), Some(height)) = (query.width, query.height) {
            image::imageops::resize(&rgba_image, width, height, image::imageops::FilterType::Lanczos3)
        } else if let Some(width) = query.width {
            let aspect_ratio = rgba_image.height() as f32 / rgba_image.width() as f32;
            let height = (width as f32 * aspect_ratio) as u32;
            image::imageops::resize(&rgba_image, width, height, image::imageops::FilterType::Lanczos3)
        } else if let Some(height) = query.height {
            let aspect_ratio = rgba_image.width() as f32 / rgba_image.height() as f32;
            let width = (height as f32 * aspect_ratio) as u32;
            image::imageops::resize(&rgba_image, width, height, image::imageops::FilterType::Lanczos3)
        } else {
            rgba_image
        };

        // Convert to PNG bytes
        let mut png_bytes = Vec::new();
        match processed_image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png) {
            Ok(_) => {},
            Err(_) => {
                return json!({
                    "content": [{
                        "type": "text",
                        "text": "Failed to encode PNG"
                    }],
                    "isError": true
                });
            }
        }

        // Encode as base64
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        json!({
            "content": [{
                "type": "image",
                "data": base64_data,
                "mimeType": "image/png"
            }]
        })
    }

    async fn handle_mouse_click_tool(&self, arguments: Value) -> Value {
        let x = arguments.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = arguments.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let button = arguments.get("button").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut enigo = self.state.enigo.lock().unwrap();
        enigo.inner.mouse_move_to(x, y);
        let mouse_button = match button.as_deref() {
            Some("right") => enigo::MouseButton::Right,
            Some("middle") => enigo::MouseButton::Middle,
            _ => enigo::MouseButton::Left,
        };
        enigo.inner.mouse_click(mouse_button);

        json!({
            "content": [{
                "type": "text",
                "text": format!("Clicked at ({}, {}) with {} button", x, y, button.as_deref().unwrap_or("left"))
            }]
        })
    }

    async fn handle_mouse_move_tool(&self, arguments: Value) -> Value {
        let x = arguments.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = arguments.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let relative = arguments.get("relative").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut enigo = self.state.enigo.lock().unwrap();
        if relative {
            enigo.inner.mouse_move_relative(x, y);
        } else {
            enigo.inner.mouse_move_to(x, y);
        }

        json!({
            "content": [{
                "type": "text",
                "text": format!("Moved mouse to ({}, {}) {}", x, y, if relative { "relatively" } else { "absolutely" })
            }]
        })
    }

    async fn handle_mouse_scroll_tool(&self, arguments: Value) -> Value {
        let x = arguments.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = arguments.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let mut enigo = self.state.enigo.lock().unwrap();
        enigo.inner.mouse_scroll_x(x);
        enigo.inner.mouse_scroll_y(y);

        json!({
            "content": [{
                "type": "text",
                "text": format!("Scrolled ({}, {})", x, y)
            }]
        })
    }

    async fn handle_type_text_tool(&self, arguments: Value) -> Value {
        let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut enigo = self.state.enigo.lock().unwrap();
        enigo.inner.key_sequence(&text);

        json!({
            "content": [{
                "type": "text",
                "text": format!("Typed: '{}'", text)
            }]
        })
    }

    async fn handle_gamepad_input_tool(&self, arguments: Value) -> Value {
        if !self.state.vigem_enabled {
            return json!({
                "content": [{
                    "type": "text",
                    "text": "ViGEm gamepad integration is disabled or not compiled in"
                }],
                "isError": true
            });
        }

        let button_a = arguments.get("button_a").and_then(|v| v.as_bool()).unwrap_or(false);
        let button_b = arguments.get("button_b").and_then(|v| v.as_bool()).unwrap_or(false);
        let left_trigger = arguments.get("left_trigger").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let right_trigger = arguments.get("right_trigger").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let left_thumb_x = arguments.get("left_thumb_x").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        let left_thumb_y = arguments.get("left_thumb_y").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        let right_thumb_x = arguments.get("right_thumb_x").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        let right_thumb_y = arguments.get("right_thumb_y").and_then(|v| v.as_i64()).unwrap_or(0) as i16;

        #[cfg(feature = "vigem")]
        {
            if let Some(target) = &*self.state.controller.lock().unwrap() {
                let report = vigem_client::X360ReportBuilder::new()
                    .button_a(button_a)
                    .button_b(button_b)
                    .trigger_left(left_trigger)
                    .trigger_right(right_trigger)
                    .thumb_lx(left_thumb_x)
                    .thumb_ly(left_thumb_y)
                    .thumb_rx(right_thumb_x)
                    .thumb_ry(right_thumb_y)
                    .build();

                match target.update(&report) {
                    Ok(_) => json!({
                        "content": [{
                            "type": "text",
                            "text": "Gamepad input sent successfully"
                        }]
                    }),
                    Err(e) => json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Failed to update gamepad: {}", e)
                        }],
                        "isError": true
                    }),
                }
            } else {
                json!({
                    "content": [{
                        "type": "text",
                        "text": "No gamepad connected"
                    }],
                    "isError": true
                })
            }
        }

        #[cfg(not(feature = "vigem"))]
        {
            json!({
                "content": [{
                    "type": "text",
                    "text": "ViGEm gamepad integration not compiled in (missing 'vigem' feature)"
                }],
                "isError": true
            })
        }
    }

    async fn handle_start_app_tool(&self, arguments: Value) -> Value {
        let exe_path = arguments.get("exe_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let args: Option<Vec<String>> = arguments.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());

        let mut proc = std::process::Command::new(&exe_path);
        if let Some(args) = &args {
            proc.args(args);
        }

        match proc.spawn() {
            Ok(child) => {
                *self.state.app_process.lock().unwrap() = Some(child);
                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Application started: {}", exe_path)
                    }]
                })
            }
            Err(e) => json!({
                "content": [{
                    "type": "text",
                    "text": format!("Failed to start app: {}", e)
                }],
                "isError": true
            }),
        }
    }

    async fn handle_stop_app_tool(&self, _arguments: Value) -> Value {
        let mut app_proc_lock = self.state.app_process.lock().unwrap();
        if let Some(mut child) = app_proc_lock.take() {
            match child.kill() {
                Ok(_) => json!({
                    "content": [{
                        "type": "text",
                        "text": "Application stopped"
                    }]
                }),
                Err(e) => json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Failed to stop app: {}", e)
                    }],
                    "isError": true
                }),
            }
        } else {
            json!({
                "content": [{
                    "type": "text",
                    "text": "No application running"
                }]
            })
        }
    }
}