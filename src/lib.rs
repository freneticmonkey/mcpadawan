// lib.rs - Library module for mcpadawan

use axum::{routing::post, routing::get, Router, Json, extract::{State, Query}};
use serde::Deserialize;
use std::{process::{Command, Child}, sync::{Arc, Mutex}};
use enigo::{Enigo, MouseControllable, KeyboardControllable, MouseButton};
use screenshots::Screen;
use image::imageops;

#[cfg(target_os = "macos")]
pub use macos::WindowInfo;
#[cfg(feature = "vigem")]
use vigem_client::{Client, TargetId, Target};

pub mod mcp;
pub mod error;
pub mod capture;

#[cfg(target_os = "macos")]
pub mod macos;


// Global flag to track if we're in MCP mode
use std::sync::OnceLock;
static MCP_MODE: OnceLock<bool> = OnceLock::new();

pub fn set_mcp_mode(enabled: bool) {
    MCP_MODE.set(enabled).ok();
}

pub fn is_mcp_mode() -> bool {
    MCP_MODE.get().copied().unwrap_or(false)
}

// Safe logging function that respects MCP mode
pub fn log_info(message: &str) {
    if is_mcp_mode() {
        // In MCP mode, use stderr to avoid polluting stdout JSON-RPC channel
        eprintln!("{}", message);
    } else {
        // In HTTP mode, use stdout normally
        println!("{}", message);
    }
}

// Thread-safe wrapper for Enigo
pub struct ThreadSafeEnigo {
    inner: Enigo,
}

unsafe impl Send for ThreadSafeEnigo {}
unsafe impl Sync for ThreadSafeEnigo {}

impl ThreadSafeEnigo {
    pub fn new() -> Self {
        ThreadSafeEnigo {
            inner: Enigo::new(),
        }
    }
}

// Struct to manage global state
#[derive(Clone)]
pub struct AppState {
    pub enigo: Arc<Mutex<ThreadSafeEnigo>>,
    pub app_process: Arc<Mutex<Option<Child>>>,
    #[cfg(feature = "vigem")]
    pub controller: Arc<Mutex<Option<Target>>>,
    pub vigem_enabled: bool,
    pub app_pid: Arc<Mutex<Option<u32>>>,
    #[cfg(target_os = "macos")]
    pub capture_manager: Arc<Mutex<capture::CaptureManager>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct StartAppRequest {
    pub exe_path: String,
    pub args: Option<Vec<String>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct ClickRequest {
    pub x: i32,
    pub y: i32,
    pub button: Option<String>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct MoveRequest {
    pub x: i32,
    pub y: i32,
    pub relative: Option<bool>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct ScrollRequest {
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize, serde::Serialize)]
pub struct TypeRequest {
    pub text: String,
}

#[derive(Deserialize, serde::Serialize)]
pub struct GamepadInputRequest {
    pub button_a: bool,
    pub button_b: bool,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_thumb_x: i16,
    pub left_thumb_y: i16,
    pub right_thumb_x: i16,
    pub right_thumb_y: i16,
}

#[derive(Deserialize, serde::Serialize)]
pub struct ScreenshotQuery {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale: Option<f32>,
}

#[cfg(target_os = "macos")]
impl AppState {
    pub fn get_windows_for_pid(&self, pid: u32) -> Result<Vec<WindowInfo>, String> {
        let mut enumerator = macos::WindowEnumerator::new();
        enumerator.find_by_pid(pid).map_err(|e| format!("Failed to enumerate windows: {}", e))
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/start_app", post(start_app))
        .route("/stop_app", post(stop_app))
        .route("/click", post(mouse_click))
        .route("/mousemove", post(mouse_move))
        .route("/scroll", post(mouse_scroll))
        .route("/type", post(keyboard_type))
        .route("/gamepad", post(gamepad_input))
        .route("/screenshot", get(screenshot))
        .with_state(state)
}

pub async fn start_app(State(state): State<AppState>, Json(payload): Json<StartAppRequest>) -> String {
    let mut proc = Command::new(&payload.exe_path);
    if let Some(args) = &payload.args {
        proc.args(args);
    }

    match proc.spawn() {
        Ok(child) => {
            let pid = child.id();
            *state.app_pid.lock().unwrap() = Some(pid);
            *state.app_process.lock().unwrap() = Some(child);
            format!("Application started with PID {}", pid)
        }
        Err(e) => format!("Failed to start app: {}", e),
    }
}

pub async fn stop_app(State(state): State<AppState>) -> String {
    let mut app_proc_lock = state.app_process.lock().unwrap();
    if let Some(mut child) = app_proc_lock.take() {
        *state.app_pid.lock().unwrap() = None;
        match child.kill() {
            Ok(_) => "Application stopped".to_string(),
            Err(e) => format!("Failed to stop app: {}", e),
        }
    } else {
        "No application running".to_string()
    }
}

pub async fn mouse_click(State(state): State<AppState>, Json(payload): Json<ClickRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.inner.mouse_move_to(payload.x, payload.y);
    let button = match payload.button.as_deref() {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    enigo.inner.mouse_click(button);
    "Clicked".to_string()
}

pub async fn mouse_move(State(state): State<AppState>, Json(payload): Json<MoveRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    if payload.relative.unwrap_or(false) {
        enigo.inner.mouse_move_relative(payload.x, payload.y);
    } else {
        enigo.inner.mouse_move_to(payload.x, payload.y);
    }
    "Moved".to_string()
}

pub async fn mouse_scroll(State(state): State<AppState>, Json(payload): Json<ScrollRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.inner.mouse_scroll_x(payload.x);
    enigo.inner.mouse_scroll_y(payload.y);
    "Scrolled".to_string()
}

pub async fn keyboard_type(State(state): State<AppState>, Json(payload): Json<TypeRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.inner.key_sequence(&payload.text);
    "Typed".to_string()
}

pub async fn screenshot(Query(params): Query<ScreenshotQuery>) -> axum::response::Response<axum::body::Body> {
    #[cfg(target_os = "macos")]
    {
        // Try native macOS capture first
        if let Ok(image) = capture_with_native_macos(&params).await {
            return image;
        }
    }
    
    // Fallback to existing implementation
    capture_full_screen_with_crop(&params, None).await
}

#[cfg(target_os = "macos")]
async fn capture_with_native_macos(params: &ScreenshotQuery) -> Result<axum::response::Response<axum::body::Body>, String> {
    use crate::capture::{CaptureManager, CaptureTarget};
    use crate::capture::native::NativeMacOSCapture;
    
    let mut manager = CaptureManager::new();
    manager.add_strategy(Box::new(NativeMacOSCapture::new()));
    
    let image = manager.capture(CaptureTarget::FullScreen)
        .map_err(|e| format!("Native capture failed: {}", e))?;
    
    let processed_image = apply_image_transformations(image, params);
    
    let mut png_bytes = Vec::new();
    processed_image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    Ok(axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(png_bytes))
        .unwrap())
}

async fn capture_full_screen_with_crop(params: &ScreenshotQuery, crop_bounds: Option<(f64, f64, f64, f64)>) -> axum::response::Response<axum::body::Body> {
    let screens = match Screen::all() {
        Ok(screens) => screens,
        Err(_) => {
            return axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from("Failed to get screens"))
                .unwrap();
        }
    };
    
    let screen = match screens.into_iter().next() {
        Some(screen) => screen,
        None => {
            return axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from("No screen found"))
                .unwrap();
        }
    };
    
    let image = match screen.capture() {
        Ok(image) => image,
        Err(_) => {
            return axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from("Failed to capture screenshot"))
                .unwrap();
        }
    };
    
    process_screenshot_image(image, params, crop_bounds)
}

// CGImage capture functions will be implemented later when 
// the core-graphics crate provides better API access

pub fn apply_image_transformations(rgba_image: image::RgbaImage, params: &ScreenshotQuery) -> image::RgbaImage {
    if let Some(scale) = params.scale {
        let new_width = (rgba_image.width() as f32 * scale) as u32;
        let new_height = (rgba_image.height() as f32 * scale) as u32;
        imageops::resize(&rgba_image, new_width, new_height, imageops::FilterType::Lanczos3)
    } else if let (Some(width), Some(height)) = (params.width, params.height) {
        imageops::resize(&rgba_image, width, height, imageops::FilterType::Lanczos3)
    } else if let Some(width) = params.width {
        let aspect_ratio = rgba_image.height() as f32 / rgba_image.width() as f32;
        let height = (width as f32 * aspect_ratio) as u32;
        imageops::resize(&rgba_image, width, height, imageops::FilterType::Lanczos3)
    } else if let Some(height) = params.height {
        let aspect_ratio = rgba_image.width() as f32 / rgba_image.height() as f32;
        let width = (height as f32 * aspect_ratio) as u32;
        imageops::resize(&rgba_image, width, height, imageops::FilterType::Lanczos3)
    } else {
        rgba_image
    }
}

fn process_screenshot_image(image: screenshots::Image, params: &ScreenshotQuery, crop_bounds: Option<(f64, f64, f64, f64)>) -> axum::response::Response<axum::body::Body> {
    let width = image.width() as u32;
    let height = image.height() as u32;
    let buffer = image.buffer();
    
    // Diagnostic information about the buffer
    let pixels_count = (width * height) as usize;
    let expected_rgba_size = pixels_count * 4;
    let bytes_per_pixel = buffer.len() as f64 / pixels_count as f64;
    
    println!("=== HTTP Screenshot Buffer Diagnostic ===");
    println!("Image dimensions: {}x{}", width, height);
    println!("Total pixels: {}", pixels_count);
    println!("Buffer length: {} bytes", buffer.len());
    println!("Expected RGBA size: {} bytes", expected_rgba_size);
    println!("Actual bytes per pixel: {:.3}", bytes_per_pixel);
    
    // Sample first few pixels for analysis
    if buffer.len() >= 12 {
        println!("First 3 pixels (raw bytes): {:?}", &buffer[0..12]);
    }
    
    // Try different approaches based on buffer size
    if buffer.len() == expected_rgba_size {
        println!("Buffer size matches RGBA expectation - trying direct conversion");
        let rgba_buffer = buffer.to_vec();
        if let Some(rgba_image) = image::RgbaImage::from_raw(width, height, rgba_buffer) {
            let processed_image = apply_image_transformations(rgba_image, params);
            return create_png_response(processed_image);
        }
    } else if buffer.len() == pixels_count * 3 {
        println!("Buffer size matches RGB expectation - converting to RGBA");
        let mut rgba_buffer = Vec::with_capacity(expected_rgba_size);
        for chunk in buffer.chunks_exact(3) {
            rgba_buffer.push(chunk[0]); // R
            rgba_buffer.push(chunk[1]); // G
            rgba_buffer.push(chunk[2]); // B
            rgba_buffer.push(255);      // A
        }
        if let Some(rgba_image) = image::RgbaImage::from_raw(width, height, rgba_buffer) {
            let processed_image = apply_image_transformations(rgba_image, params);
            return create_png_response(processed_image);
        }
    } else {
        println!("Unexpected buffer size - checking for compression or different format");
        
        // Check if it might be compressed data (PNG signature or unusual size)
        let has_png_signature = buffer.len() >= 8 && 
            buffer[0] == 137 && buffer[1] == 80 && buffer[2] == 78 && buffer[3] == 71;
        
        if buffer.len() < pixels_count || has_png_signature {
            if has_png_signature {
                println!("Buffer has PNG signature - attempting PNG decode");
            } else {
                println!("Buffer appears to be compressed (smaller than pixel count)");
            }
            
            // Try to decode as PNG
            println!("Attempting to decode as PNG...");
            match image::load_from_memory(buffer) {
                Ok(decoded_image) => {
                    println!("Successfully decoded buffer as compressed image format: {:?}", decoded_image.color());
                    let rgba_image = decoded_image.to_rgba8();
                    let processed_image = apply_image_transformations(rgba_image, params);
                    return create_png_response(processed_image);
                },
                Err(e) => {
                    println!("Failed to decode as PNG: {:?}", e);
                }
            }
            
            // Try other common formats
            for format in [image::ImageFormat::Jpeg, image::ImageFormat::WebP, image::ImageFormat::Bmp] {
                if let Ok(decoded_image) = image::load_from_memory_with_format(buffer, format) {
                    println!("Successfully decoded buffer as {:?}", format);
                    let rgba_image = decoded_image.to_rgba8();
                    let processed_image = apply_image_transformations(rgba_image, params);
                    return create_png_response(processed_image);
                }
            }
        }
    }
    
    println!("=== End HTTP Diagnostic ===");
    
    return axum::response::Response::builder()
        .status(500)
        .body(axum::body::Body::from(format!("Failed to process screenshot buffer: {}x{}, buffer len: {}, bytes per pixel: {:.3}", 
                                           width, height, buffer.len(), bytes_per_pixel)))
        .unwrap();
}

fn create_png_response(processed_image: image::RgbaImage) -> axum::response::Response<axum::body::Body> {
    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    match processed_image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png) {
        Ok(_) => {},
        Err(_) => {
            return axum::response::Response::builder()
                .status(500)
                .body(axum::body::Body::from("Failed to encode PNG"))
                .unwrap();
        }
    }
    
    axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(png_bytes))
        .unwrap()
}

pub async fn gamepad_input(State(state): State<AppState>, Json(_input): Json<GamepadInputRequest>) -> String {
    if !state.vigem_enabled {
        return "ViGEm gamepad integration is disabled or not compiled in".to_string();
    }
    
    #[cfg(feature = "vigem")]
    {
        if let Some(target) = &*state.controller.lock().unwrap() {
            let report = vigem_client::X360ReportBuilder::new()
                .button_a(_input.button_a)
                .button_b(_input.button_b)
                .trigger_left(_input.left_trigger)
                .trigger_right(_input.right_trigger)
                .thumb_lx(_input.left_thumb_x)
                .thumb_ly(_input.left_thumb_y)
                .thumb_rx(_input.right_thumb_x)
                .thumb_ry(_input.right_thumb_y)
                .build();

            match target.update(&report) {
                Ok(_) => "Gamepad input sent".to_string(),
                Err(e) => format!("Failed to update gamepad: {}", e),
            }
        } else {
            "No gamepad connected".to_string()
        }
    }
    
    #[cfg(not(feature = "vigem"))]
    {
        "ViGEm gamepad integration not compiled in (missing 'vigem' feature)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use std::sync::Arc;
    
    fn create_test_state() -> AppState {
        AppState {
            enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
            app_process: Arc::new(Mutex::new(None)),
            #[cfg(feature = "vigem")]
            controller: Arc::new(Mutex::new(None)),
            vigem_enabled: false,
            app_pid: Arc::new(Mutex::new(None)),
            #[cfg(target_os = "macos")]
            capture_manager: Arc::new(Mutex::new(capture::CaptureManager::new())),
        }
    }

    #[tokio::test]
    async fn test_screenshot_query_parsing() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        // Test without parameters - should not fail parsing
        let response = server.get("/screenshot").await;
        assert!(response.status_code().is_success() || response.status_code() == 500); // 500 is ok for screenshot failure in test env
        
        // Test with scale parameter
        let response = server.get("/screenshot?scale=0.5").await;
        assert!(response.status_code().is_success() || response.status_code() == 500);
        
        // Test with width parameter
        let response = server.get("/screenshot?width=800").await;
        assert!(response.status_code().is_success() || response.status_code() == 500);
        
        // Test with height parameter
        let response = server.get("/screenshot?height=600").await;
        assert!(response.status_code().is_success() || response.status_code() == 500);
        
        // Test with width and height parameters
        let response = server.get("/screenshot?width=800&height=600").await;
        assert!(response.status_code().is_success() || response.status_code() == 500);
    }

    #[tokio::test]
    async fn test_mouse_click_endpoint() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let click_data = ClickRequest {
            x: 100,
            y: 200,
            button: Some("left".to_string()),
        };

        let response = server.post("/click").json(&click_data).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Clicked");
    }

    #[tokio::test]
    async fn test_mouse_move_endpoint() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let move_data = MoveRequest {
            x: 100,
            y: 200,
            relative: Some(false),
        };

        let response = server.post("/mousemove").json(&move_data).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Moved");
    }

    #[tokio::test]
    async fn test_keyboard_type_endpoint() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let type_data = TypeRequest {
            text: "Hello, world!".to_string(),
        };

        let response = server.post("/type").json(&type_data).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Typed");
    }

    #[tokio::test]
    async fn test_scroll_endpoint() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let scroll_data = ScrollRequest {
            x: 0,
            y: 3,
        };

        let response = server.post("/scroll").json(&scroll_data).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Scrolled");
    }

    #[tokio::test]
    async fn test_gamepad_endpoint_disabled() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let gamepad_data = GamepadInputRequest {
            button_a: true,
            button_b: false,
            left_trigger: 128,
            right_trigger: 0,
            left_thumb_x: 1000,
            left_thumb_y: 0,
            right_thumb_x: 0,
            right_thumb_y: 0,
        };

        let response = server.post("/gamepad").json(&gamepad_data).await;
        assert_eq!(response.status_code(), 200);
        assert!(response.text().contains("disabled") || response.text().contains("not compiled in"));
    }

    #[tokio::test]
    async fn test_stop_app_no_process() {
        let state = create_test_state();
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let response = server.post("/stop_app").await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "No application running");
    }
}