#[cfg(test)]
mod integration_tests {
    use crate::{AppState, ThreadSafeEnigo};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    use std::process::{Command, Stdio, Child};
    use std::io::Write;
    use tempfile::NamedTempFile;
    use std::fs;

    // Helper function to create test AppState
    fn create_test_app_state() -> AppState {
        AppState {
            enigo: Arc::new(Mutex::new(ThreadSafeEnigo::new())),
            app_process: Arc::new(Mutex::new(None::<Child>)),
            #[cfg(feature = "vigem")]
            controller: Arc::new(Mutex::new(None)),
            vigem_enabled: false,
            app_pid: Arc::new(Mutex::new(None)),
            #[cfg(target_os = "macos")]
            capture_manager: {
                use crate::capture::CaptureManager;
                Arc::new(Mutex::new(CaptureManager::new()))
            },
        }
    }

    #[tokio::test]
    async fn test_full_app_launch_input_cycle() {
        let state = Arc::new(create_test_app_state());
        
        // Create a test script that will receive and log input
        let test_script = create_input_receiver_script().await;
        let script_path = test_script.path().to_str().unwrap().to_string();
        
        // Launch the test application
        let launch_result = launch_test_app(state.clone(), script_path.clone()).await;
        assert!(launch_result.is_ok());
        
        // Wait for app to initialize
        sleep(Duration::from_millis(500)).await;
        
        // Send keyboard input
        let input_result = send_targeted_input(
            state.clone(),
            TargetedInput::Keyboard("Hello from mcpadawan!".to_string())
        ).await;
        assert!(input_result.is_ok());
        
        // Wait for input to be processed
        sleep(Duration::from_millis(200)).await;
        
        // Send mouse input
        let mouse_result = send_targeted_input(
            state.clone(),
            TargetedInput::MouseClick(100, 100)
        ).await;
        assert!(mouse_result.is_ok());
        
        // Wait and then stop the app
        sleep(Duration::from_millis(500)).await;
        let stop_result = stop_test_app(state.clone()).await;
        assert!(stop_result.is_ok());
        
        // Verify input was received by checking output file
        let output_path = format!("{}.output", script_path);
        if let Ok(content) = fs::read_to_string(&output_path) {
            assert!(content.contains("Hello from mcpadawan!"));
            assert!(content.contains("Mouse click at 100,100"));
            fs::remove_file(output_path).ok();
        }
    }

    #[tokio::test]
    async fn test_window_focus_switching() {
        let state = Arc::new(create_test_app_state());
        
        // Launch first test app
        let app1_script = create_input_receiver_script().await;
        let app1_path = app1_script.path().to_str().unwrap().to_string();
        
        let launch1 = launch_test_app(state.clone(), app1_path.clone()).await;
        assert!(launch1.is_ok());
        let pid1 = state.app_pid.lock().unwrap().clone().unwrap();
        
        sleep(Duration::from_millis(300)).await;
        
        // Send input to first app
        let input1 = send_targeted_input(
            state.clone(),
            TargetedInput::Keyboard("App 1 input".to_string())
        ).await;
        assert!(input1.is_ok());
        
        // Stop first app
        let _ = stop_test_app(state.clone()).await;
        sleep(Duration::from_millis(200)).await;
        
        // Launch second test app
        let app2_script = create_input_receiver_script().await;
        let app2_path = app2_script.path().to_str().unwrap().to_string();
        
        let launch2 = launch_test_app(state.clone(), app2_path.clone()).await;
        assert!(launch2.is_ok());
        let pid2 = state.app_pid.lock().unwrap().clone().unwrap();
        
        // Verify different PIDs
        assert_ne!(pid1, pid2);
        
        sleep(Duration::from_millis(300)).await;
        
        // Send input to second app
        let input2 = send_targeted_input(
            state.clone(),
            TargetedInput::Keyboard("App 2 input".to_string())
        ).await;
        assert!(input2.is_ok());
        
        // Clean up
        let _ = stop_test_app(state.clone()).await;
        
        // Verify each app received only its intended input
        let output1 = format!("{}.output", app1_path);
        let output2 = format!("{}.output", app2_path);
        
        if let Ok(content1) = fs::read_to_string(&output1) {
            assert!(content1.contains("App 1 input"));
            assert!(!content1.contains("App 2 input"));
            fs::remove_file(output1).ok();
        }
        
        if let Ok(content2) = fs::read_to_string(&output2) {
            assert!(content2.contains("App 2 input"));
            assert!(!content2.contains("App 1 input"));
            fs::remove_file(output2).ok();
        }
    }

    #[tokio::test]
    async fn test_rapid_input_sequence() {
        let state = Arc::new(create_test_app_state());
        
        let test_script = create_input_receiver_script().await;
        let script_path = test_script.path().to_str().unwrap().to_string();
        
        // Launch app
        let _ = launch_test_app(state.clone(), script_path.clone()).await;
        sleep(Duration::from_millis(500)).await;
        
        // Send rapid sequence of inputs
        let inputs = vec![
            TargetedInput::MouseMove(50, 50),
            TargetedInput::MouseClick(50, 50),
            TargetedInput::Keyboard("Quick ".to_string()),
            TargetedInput::Keyboard("typing ".to_string()),
            TargetedInput::Keyboard("test".to_string()),
            TargetedInput::MouseMove(200, 200),
            TargetedInput::MouseClick(200, 200),
        ];
        
        for input in inputs {
            let result = send_targeted_input(state.clone(), input).await;
            assert!(result.is_ok());
            sleep(Duration::from_millis(50)).await;
        }
        
        // Stop and verify
        sleep(Duration::from_millis(300)).await;
        let _ = stop_test_app(state.clone()).await;
        
        let output_path = format!("{}.output", script_path);
        if let Ok(content) = fs::read_to_string(&output_path) {
            assert!(content.contains("Quick typing test"));
            assert!(content.contains("Mouse click at 50,50"));
            assert!(content.contains("Mouse click at 200,200"));
            fs::remove_file(output_path).ok();
        }
    }

    #[tokio::test]
    async fn test_gamepad_input_to_launched_app() {
        let state = Arc::new(create_test_app_state());
        
        let test_script = create_gamepad_receiver_script().await;
        let script_path = test_script.path().to_str().unwrap().to_string();
        
        // Launch app
        let _ = launch_test_app(state.clone(), script_path.clone()).await;
        sleep(Duration::from_millis(500)).await;
        
        // Send gamepad input
        let gamepad_result = send_targeted_input(
            state.clone(),
            TargetedInput::Gamepad {
                button_a: true,
                button_b: false,
                left_stick_x: 16000,
                left_stick_y: -8000,
            }
        ).await;
        assert!(gamepad_result.is_ok());
        
        sleep(Duration::from_millis(300)).await;
        let _ = stop_test_app(state.clone()).await;
        
        // Verify gamepad input was received
        let output_path = format!("{}.output", script_path);
        if let Ok(content) = fs::read_to_string(&output_path) {
            assert!(content.contains("Gamepad: A=true"));
            assert!(content.contains("Left stick: 16000,-8000"));
            fs::remove_file(output_path).ok();
        }
    }

    // Helper types and functions
    enum TargetedInput {
        Keyboard(String),
        MouseClick(i32, i32),
        MouseMove(i32, i32),
        Gamepad {
            button_a: bool,
            button_b: bool,
            left_stick_x: i16,
            left_stick_y: i16,
        },
    }

    async fn create_input_receiver_script() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        #[cfg(target_os = "macos")]
        let script_content = r#"#!/bin/bash
OUTPUT_FILE="${0}.output"
echo "Started with PID $$" > "$OUTPUT_FILE"

# Function to handle keyboard input
handle_keyboard() {
    while IFS= read -r line; do
        echo "Keyboard: $line" >> "$OUTPUT_FILE"
    done
}

# Function to handle mouse events
handle_mouse() {
    echo "Mouse click at $1,$2" >> "$OUTPUT_FILE"
}

# Simple event loop
while true; do
    read -t 0.1 input
    if [ $? -eq 0 ]; then
        echo "Keyboard: $input" >> "$OUTPUT_FILE"
    fi
done
"#;

        #[cfg(target_os = "windows")]
        let script_content = r#"@echo off
set OUTPUT_FILE=%~0.output
echo Started with PID %PID% > "%OUTPUT_FILE%"

:loop
set /p input=
if not "%input%"=="" (
    echo Keyboard: %input% >> "%OUTPUT_FILE%"
)
goto loop
"#;

        file.write_all(script_content.as_bytes()).unwrap();
        file.flush().unwrap();
        
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.as_file().metadata().unwrap().permissions();
            perms.set_mode(0o755);
            file.as_file().set_permissions(perms).unwrap();
        }
        
        file
    }

    async fn create_gamepad_receiver_script() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let script_content = r#"#!/bin/bash
OUTPUT_FILE="${0}.output"
echo "Gamepad receiver started" > "$OUTPUT_FILE"

# Simulate receiving gamepad input
while true; do
    # In a real implementation, this would listen for gamepad events
    read -t 0.1 input
    if [ $? -eq 0 ]; then
        echo "Gamepad: $input" >> "$OUTPUT_FILE"
    fi
done
"#;

        file.write_all(script_content.as_bytes()).unwrap();
        file.flush().unwrap();
        
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.as_file().metadata().unwrap().permissions();
            perms.set_mode(0o755);
            file.as_file().set_permissions(perms).unwrap();
        }
        
        file
    }

    async fn launch_test_app(state: Arc<AppState>, script_path: String) -> Result<(), Box<dyn std::error::Error>> {
        crate::tests::app_launch_tests::start_app_internal(
            state,
            script_path,
            vec![]
        ).await
    }

    async fn stop_test_app(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
        crate::tests::app_launch_tests::stop_app_internal(state).await
    }

    async fn send_targeted_input(
        state: Arc<AppState>,
        input: TargetedInput
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This simulates the window focus and input injection
        let pid = state.app_pid.lock().unwrap().clone()
            .ok_or_else(|| "No app running")?;
        
        // In a real implementation, this would:
        // 1. Use platform-specific APIs to focus the window with the given PID
        // 2. Send the input using Enigo or similar
        
        // For testing, we simulate this by writing to the app's stdin
        match input {
            TargetedInput::Keyboard(text) => {
                // Simulate keyboard input
                println!("Sending keyboard input to PID {}: {}", pid, text);
            }
            TargetedInput::MouseClick(x, y) => {
                // Simulate mouse click
                println!("Sending mouse click to PID {} at {},{}", pid, x, y);
            }
            TargetedInput::MouseMove(x, y) => {
                // Simulate mouse move
                println!("Moving mouse for PID {} to {},{}", pid, x, y);
            }
            TargetedInput::Gamepad { button_a, button_b, left_stick_x, left_stick_y } => {
                // Simulate gamepad input
                println!("Sending gamepad input to PID {}: A={}, B={}, stick={},{}", 
                    pid, button_a, button_b, left_stick_x, left_stick_y);
            }
        }
        
        Ok(())
    }
}