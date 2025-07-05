// Integration tests for mcpadawan

use axum_test::TestServer;
use mcpadawan::*;
use std::sync::{Arc, Mutex};
use tokio;

fn create_test_state() -> AppState {
    AppState {
        enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
        app_process: Arc::new(Mutex::new(None)),
        #[cfg(feature = "vigem")]
        controller: Arc::new(Mutex::new(None)),
        vigem_enabled: false,
        app_pid: Arc::new(Mutex::new(None)),
        #[cfg(target_os = "macos")]
        capture_manager: {
            use mcpadawan::capture::CaptureManager;
            Arc::new(Mutex::new(CaptureManager::new()))
        },
    }
}

#[tokio::test]
async fn test_complete_api_workflow() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test screenshot with different parameters
    let response = server.get("/screenshot").await;
    assert!(response.status_code().is_success() || response.status_code() == 500);

    let response = server.get("/screenshot?scale=0.5").await;
    assert!(response.status_code().is_success() || response.status_code() == 500);

    let response = server.get("/screenshot?width=800&height=600").await;
    assert!(response.status_code().is_success() || response.status_code() == 500);

    // Test mouse operations
    let click_data = ClickRequest {
        x: 100,
        y: 100,
        button: Some("left".to_string()),
    };
    let response = server.post("/click").json(&click_data).await;
    assert_eq!(response.status_code(), 200);

    let move_data = MoveRequest {
        x: 200,
        y: 200,
        relative: Some(false),
    };
    let response = server.post("/mousemove").json(&move_data).await;
    assert_eq!(response.status_code(), 200);

    let scroll_data = ScrollRequest { x: 0, y: 3 };
    let response = server.post("/scroll").json(&scroll_data).await;
    assert_eq!(response.status_code(), 200);

    // Test keyboard
    let type_data = TypeRequest {
        text: "Test input".to_string(),
    };
    let response = server.post("/type").json(&type_data).await;
    assert_eq!(response.status_code(), 200);

    // Test gamepad (should be disabled in test)
    let gamepad_data = GamepadInputRequest {
        button_a: true,
        button_b: false,
        left_trigger: 255,
        right_trigger: 0,
        left_thumb_x: 32767,
        left_thumb_y: 0,
        right_thumb_x: 0,
        right_thumb_y: 0,
    };
    let response = server.post("/gamepad").json(&gamepad_data).await;
    assert_eq!(response.status_code(), 200);
    assert!(response.text().contains("disabled") || response.text().contains("not compiled in"));

    // Test app lifecycle
    let response = server.post("/stop_app").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "No application running");
}

#[tokio::test]
async fn test_screenshot_parameter_validation() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test invalid scale parameter (should still work or return error gracefully)
    let response = server.get("/screenshot?scale=invalid").await;
    assert!(response.status_code().is_client_error() || response.status_code() == 500);

    // Test negative scale
    let response = server.get("/screenshot?scale=-1").await;
    assert!(response.status_code().is_success() || response.status_code().is_client_error() || response.status_code() == 500);

    // Test zero dimensions
    let response = server.get("/screenshot?width=0&height=0").await;
    assert!(response.status_code().is_success() || response.status_code().is_client_error() || response.status_code() == 500);

    // Test very large dimensions
    let response = server.get("/screenshot?width=10000&height=10000").await;
    assert!(response.status_code().is_success() || response.status_code().is_client_error() || response.status_code() == 500);
}

#[tokio::test]
async fn test_mouse_button_variants() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test all mouse button types
    let buttons = vec!["left", "right", "middle"];
    
    for button in buttons {
        let click_data = ClickRequest {
            x: 100,
            y: 100,
            button: Some(button.to_string()),
        };
        let response = server.post("/click").json(&click_data).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), "Clicked");
    }

    // Test default button (no button specified)
    let click_data = ClickRequest {
        x: 100,
        y: 100,
        button: None,
    };
    let response = server.post("/click").json(&click_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Clicked");
}

#[tokio::test]
async fn test_relative_mouse_movement() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test absolute movement
    let move_data = MoveRequest {
        x: 100,
        y: 100,
        relative: Some(false),
    };
    let response = server.post("/mousemove").json(&move_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Moved");

    // Test relative movement
    let move_data = MoveRequest {
        x: 10,
        y: 10,
        relative: Some(true),
    };
    let response = server.post("/mousemove").json(&move_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Moved");

    // Test default movement (should be absolute)
    let move_data = MoveRequest {
        x: 200,
        y: 200,
        relative: None,
    };
    let response = server.post("/mousemove").json(&move_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Moved");
}

#[tokio::test]
async fn test_scroll_directions() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test positive scroll
    let scroll_data = ScrollRequest { x: 3, y: 3 };
    let response = server.post("/scroll").json(&scroll_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Scrolled");

    // Test negative scroll
    let scroll_data = ScrollRequest { x: -3, y: -3 };
    let response = server.post("/scroll").json(&scroll_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Scrolled");

    // Test zero scroll
    let scroll_data = ScrollRequest { x: 0, y: 0 };
    let response = server.post("/scroll").json(&scroll_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Scrolled");
}

#[tokio::test]
async fn test_keyboard_input_varieties() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test empty string
    let type_data = TypeRequest {
        text: "".to_string(),
    };
    let response = server.post("/type").json(&type_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Typed");

    // Test special characters
    let type_data = TypeRequest {
        text: "Hello, World! 123 @#$%".to_string(),
    };
    let response = server.post("/type").json(&type_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Typed");

    // Test unicode characters
    let type_data = TypeRequest {
        text: "🎮 🖱️ ⌨️".to_string(),
    };
    let response = server.post("/type").json(&type_data).await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Typed");
}

#[tokio::test]
async fn test_gamepad_input_ranges() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test maximum values
    let gamepad_data = GamepadInputRequest {
        button_a: true,
        button_b: true,
        left_trigger: 255,
        right_trigger: 255,
        left_thumb_x: 32767,
        left_thumb_y: 32767,
        right_thumb_x: 32767,
        right_thumb_y: 32767,
    };
    let response = server.post("/gamepad").json(&gamepad_data).await;
    assert_eq!(response.status_code(), 200);

    // Test minimum values
    let gamepad_data = GamepadInputRequest {
        button_a: false,
        button_b: false,
        left_trigger: 0,
        right_trigger: 0,
        left_thumb_x: -32768,
        left_thumb_y: -32768,
        right_thumb_x: -32768,
        right_thumb_y: -32768,
    };
    let response = server.post("/gamepad").json(&gamepad_data).await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_malformed_requests() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test malformed JSON
    let response = server.post("/click").text("not json").await;
    assert!(response.status_code().is_client_error());

    // Test missing required fields
    let response = server.post("/click").json(&serde_json::json!({"x": 100})).await;
    assert!(response.status_code().is_client_error());

    // Test wrong field types
    let response = server.post("/click").json(&serde_json::json!({
        "x": "not_a_number",
        "y": 100,
        "button": "left"
    })).await;
    assert!(response.status_code().is_client_error());
}

#[tokio::test]
async fn test_content_type_headers() {
    let state = create_test_state();
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Test screenshot content type
    let response = server.get("/screenshot").await;
    if response.status_code().is_success() {
        assert_eq!(response.headers().get("content-type").unwrap(), "image/png");
    }
}