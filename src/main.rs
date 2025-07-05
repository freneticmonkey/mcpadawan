// mcp_server/src/main.rs

use axum::{routing::post, routing::get, Router, Json, extract::{State, Query}};
use serde::Deserialize;
use std::{process::{Command, Child}, sync::{Arc, Mutex}};
use std::net::SocketAddr;
use enigo::{Enigo, MouseControllable, KeyboardControllable, MouseButton};

// Thread-safe wrapper for Enigo
struct ThreadSafeEnigo {
    inner: Enigo,
}

unsafe impl Send for ThreadSafeEnigo {}
unsafe impl Sync for ThreadSafeEnigo {}

impl ThreadSafeEnigo {
    fn new() -> Self {
        ThreadSafeEnigo {
            inner: Enigo::new(),
        }
    }
}
use screenshots::Screen;
use image::{ImageBuffer, RgbaImage, imageops};
#[cfg(feature = "vigem")]
use vigem_client::{Client, TargetId, Target};
use clap::Parser;
use std::fs;
use std::path::Path;

// Command line arguments
#[derive(Parser)]
#[command(name = "mcpadawan")]
#[command(about = "A remote control server for applications")]
struct Args {
    /// Disable ViGEm gamepad integration
    #[arg(long, help = "Disable ViGEm gamepad integration")]
    disable_vigem: bool,
    
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml", help = "Path to configuration file")]
    config: String,
}

// Configuration structure
#[derive(Deserialize, Default)]
struct Config {
    #[serde(default)]
    disable_vigem: bool,
}

// Struct to manage global state
#[derive(Clone)]
struct AppState {
    enigo: Arc<Mutex<ThreadSafeEnigo>>,
    app_process: Arc<Mutex<Option<Child>>>,
    #[cfg(feature = "vigem")]
    controller: Arc<Mutex<Option<Target>>>,
    vigem_enabled: bool,
}

#[derive(Deserialize)]
struct StartAppRequest {
    exe_path: String,
    args: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ClickRequest {
    x: i32,
    y: i32,
    button: Option<String>,
}

#[derive(Deserialize)]
struct MoveRequest {
    x: i32,
    y: i32,
    relative: Option<bool>,
}

#[derive(Deserialize)]
struct ScrollRequest {
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct TypeRequest {
    text: String,
}

#[derive(Deserialize)]
struct GamepadInputRequest {
    button_a: bool,
    button_b: bool,
    left_trigger: u8,
    right_trigger: u8,
    left_thumb_x: i16,
    left_thumb_y: i16,
    right_thumb_x: i16,
    right_thumb_y: i16,
}

#[derive(Deserialize)]
struct ScreenshotQuery {
    width: Option<u32>,
    height: Option<u32>,
    scale: Option<f32>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // Load configuration file
    let config: Config = if Path::new(&args.config).exists() {
        let config_content = fs::read_to_string(&args.config)
            .expect("Failed to read configuration file");
        toml::from_str(&config_content)
            .expect("Failed to parse configuration file")
    } else {
        Config::default()
    };
    
    // Determine if ViGEm should be disabled (CLI flag takes precedence)
    let vigem_disabled = args.disable_vigem || config.disable_vigem;
    
    // Initialize ViGEm controller if not disabled
    #[cfg(feature = "vigem")]
    let controller = if vigem_disabled {
        println!("ViGEm gamepad integration disabled");
        None
    } else {
        match Client::connect() {
            Ok(client) => {
                let target = Target::new(TargetId::X360);
                match client.plugin(&target) {
                    Ok(_) => {
                        println!("ViGEm gamepad integration enabled");
                        Some(target)
                    }
                    Err(e) => {
                        println!("Failed to plug in virtual controller: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to ViGEmBus: {}", e);
                None
            }
        }
    };
    
    #[cfg(not(feature = "vigem"))]
    let _controller = {
        if !vigem_disabled {
            println!("ViGEm gamepad integration not compiled in (missing 'vigem' feature)");
        } else {
            println!("ViGEm gamepad integration disabled");
        }
    };

    let state = AppState {
        enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
        app_process: Arc::new(Mutex::new(None)),
        #[cfg(feature = "vigem")]
        controller: Arc::new(Mutex::new(controller)),
        vigem_enabled: !vigem_disabled && cfg!(feature = "vigem"),
    };

    let app = Router::new()
        .route("/start_app", post(start_app))
        .route("/stop_app", post(stop_app))
        .route("/click", post(mouse_click))
        .route("/mousemove", post(mouse_move))
        .route("/scroll", post(mouse_scroll))
        .route("/type", post(keyboard_type))
        .route("/gamepad", post(gamepad_input))
        .route("/screenshot", get(screenshot))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn start_app(State(state): State<AppState>, Json(payload): Json<StartAppRequest>) -> String {
    let mut proc = Command::new(&payload.exe_path);
    if let Some(args) = &payload.args {
        proc.args(args);
    }

    match proc.spawn() {
        Ok(child) => {
            *state.app_process.lock().unwrap() = Some(child);
            "Application started".to_string()
        }
        Err(e) => format!("Failed to start app: {}", e),
    }
}

async fn stop_app(State(state): State<AppState>) -> String {
    let mut app_proc_lock = state.app_process.lock().unwrap();
    if let Some(mut child) = app_proc_lock.take() {
        match child.kill() {
            Ok(_) => "Application stopped".to_string(),
            Err(e) => format!("Failed to stop app: {}", e),
        }
    } else {
        "No application running".to_string()
    }
}

async fn mouse_click(State(state): State<AppState>, Json(payload): Json<ClickRequest>) -> String {
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

async fn mouse_move(State(state): State<AppState>, Json(payload): Json<MoveRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    if payload.relative.unwrap_or(false) {
        enigo.inner.mouse_move_relative(payload.x, payload.y);
    } else {
        enigo.inner.mouse_move_to(payload.x, payload.y);
    }
    "Moved".to_string()
}

async fn mouse_scroll(State(state): State<AppState>, Json(payload): Json<ScrollRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.inner.mouse_scroll_x(payload.x);
    enigo.inner.mouse_scroll_y(payload.y);
    "Scrolled".to_string()
}

async fn keyboard_type(State(state): State<AppState>, Json(payload): Json<TypeRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.inner.key_sequence(&payload.text);
    "Typed".to_string()
}

async fn screenshot(Query(params): Query<ScreenshotQuery>) -> axum::response::Response<axum::body::Body> {
    let screens = Screen::all().unwrap();
    let screen = screens.into_iter().next().unwrap();
    let image = screen.capture().unwrap();
    
    // Convert screenshot to RgbaImage
    let rgba_image = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        image.width() as u32,
        image.height() as u32,
        image.buffer().to_vec(),
    ).unwrap();
    
    // Apply scaling or resize based on parameters
    let processed_image = if let Some(scale) = params.scale {
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
    };
    
    // Convert back to PNG bytes
    let mut png_bytes = Vec::new();
    processed_image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();
    
    axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Body::from(png_bytes))
        .unwrap()
}

async fn gamepad_input(State(state): State<AppState>, Json(_input): Json<GamepadInputRequest>) -> String {
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