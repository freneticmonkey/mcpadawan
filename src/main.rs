// mcp_server/src/main.rs

use axum::{routing::post, routing::get, Router, Json, extract::State};
use serde::{Deserialize, Serialize};
use std::{process::{Command, Child}, sync::{Arc, Mutex}};
use std::net::SocketAddr;
use enigo::{Enigo, MouseControllable, KeyboardControllable, Key, MouseButton};
use screenshots::Screen;
use tokio::task;
use vigem_client::{Client, TargetId, Target};

// Struct to manage global state
#[derive(Clone)]
struct AppState {
    enigo: Arc<Mutex<Enigo>>,
    app_process: Arc<Mutex<Option<Child>>>,
    controller: Arc<Mutex<Option<Target>>>,
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

#[tokio::main]
async fn main() {
    let client = Client::connect().expect("Failed to connect to ViGEmBus");
    let target = Target::new(TargetId::X360);
    client.plugin(&target).expect("Failed to plug in virtual controller");

    let state = AppState {
        enigo: Arc::new(Mutex::new(Enigo::new())),
        app_process: Arc::new(Mutex::new(None)),
        controller: Arc::new(Mutex::new(Some(target))),
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
    enigo.mouse_move_to(payload.x, payload.y);
    let button = match payload.button.as_deref() {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    enigo.mouse_click(button);
    "Clicked".to_string()
}

async fn mouse_move(State(state): State<AppState>, Json(payload): Json<MoveRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    if payload.relative.unwrap_or(false) {
        enigo.mouse_move_relative(payload.x, payload.y);
    } else {
        enigo.mouse_move_to(payload.x, payload.y);
    }
    "Moved".to_string()
}

async fn mouse_scroll(State(state): State<AppState>, Json(payload): Json<ScrollRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.mouse_scroll_x(payload.x);
    enigo.mouse_scroll_y(payload.y);
    "Scrolled".to_string()
}

async fn keyboard_type(State(state): State<AppState>, Json(payload): Json<TypeRequest>) -> String {
    let mut enigo = state.enigo.lock().unwrap();
    enigo.key_sequence(&payload.text);
    "Typed".to_string()
}

async fn screenshot() -> axum::response::Response {
    let screens = Screen::all().unwrap();
    let screen = screens.into_iter().next().unwrap();
    let image = screen.capture().unwrap();
    let buffer = image.to_png().unwrap();
    axum::response::Response::builder()
        .header("Content-Type", "image/png")
        .body(axum::body::Bytes::from(buffer))
        .unwrap()
}

async fn gamepad_input(State(state): State<AppState>, Json(input): Json<GamepadInputRequest>) -> String {
    if let Some(target) = &*state.controller.lock().unwrap() {
        let report = vigem_client::X360ReportBuilder::new()
            .button_a(input.button_a)
            .button_b(input.button_b)
            .trigger_left(input.left_trigger)
            .trigger_right(input.right_trigger)
            .thumb_lx(input.left_thumb_x)
            .thumb_ly(input.left_thumb_y)
            .thumb_rx(input.right_thumb_x)
            .thumb_ry(input.right_thumb_y)
            .build();

        match target.update(&report) {
            Ok(_) => "Gamepad input sent".to_string(),
            Err(e) => format!("Failed to update gamepad: {}", e),
        }
    } else {
        "No gamepad connected".to_string()
    }
}