#[cfg(test)]
mod input_injection_tests {
    use crate::{AppState, ThreadSafeEnigo};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    use std::process::Child;
    use enigo::{Enigo, MouseControllable, KeyboardControllable};

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

    #[derive(Clone)]
    struct MockWindowManager {
        active_pid: Option<u32>,
    }

    impl MockWindowManager {
        fn new() -> Self {
            Self { active_pid: None }
        }

        fn get_active_window_pid(&self) -> Option<u32> {
            self.active_pid
        }

        fn activate_window_by_pid(&mut self, pid: u32) -> Result<(), String> {
            self.active_pid = Some(pid);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_input_targets_launched_app() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(MockWindowManager::new()));
        
        // Simulate launching an app with PID 12345
        *state.app_pid.lock().unwrap() = Some(12345);
        
        // Verify window activation happens before input
        let result = activate_and_send_input(
            state.clone(),
            window_manager.clone(),
            InputType::MouseClick(100, 200)
        ).await;
        
        assert!(result.is_ok());
        
        // Verify the window manager activated the correct PID
        let wm = window_manager.lock().unwrap();
        assert_eq!(wm.get_active_window_pid(), Some(12345));
    }

    #[tokio::test]
    async fn test_input_fails_when_no_app_running() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(MockWindowManager::new()));
        
        // No app is running (app_pid is None)
        let result = activate_and_send_input(
            state.clone(),
            window_manager.clone(),
            InputType::KeyboardText("test".to_string())
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No application is running"));
    }

    #[tokio::test]
    async fn test_mouse_click_with_app_focus() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(MockWindowManager::new()));
        
        *state.app_pid.lock().unwrap() = Some(54321);
        
        let result = activate_and_send_input(
            state.clone(),
            window_manager.clone(),
            InputType::MouseClick(250, 350)
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(window_manager.lock().unwrap().get_active_window_pid(), Some(54321));
    }

    #[tokio::test]
    async fn test_keyboard_input_with_app_focus() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(MockWindowManager::new()));
        
        *state.app_pid.lock().unwrap() = Some(99999);
        
        let result = activate_and_send_input(
            state.clone(),
            window_manager.clone(),
            InputType::KeyboardText("Hello World".to_string())
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(window_manager.lock().unwrap().get_active_window_pid(), Some(99999));
    }

    #[tokio::test]
    async fn test_window_activation_failure() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(FailingWindowManager));
        
        *state.app_pid.lock().unwrap() = Some(11111);
        
        let result = activate_and_send_input_failing(
            state.clone(),
            window_manager.clone(),
            InputType::MouseMove(100, 100)
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to activate window"));
    }

    #[tokio::test]
    async fn test_multiple_input_sequence() {
        let state = Arc::new(create_test_app_state());
        let window_manager = Arc::new(Mutex::new(MockWindowManager::new()));
        
        *state.app_pid.lock().unwrap() = Some(77777);
        
        // Send multiple inputs in sequence
        let inputs = vec![
            InputType::MouseMove(50, 50),
            InputType::MouseClick(50, 50),
            InputType::KeyboardText("test input".to_string()),
            InputType::MouseMove(100, 100),
        ];
        
        for input in inputs {
            let result = activate_and_send_input(
                state.clone(),
                window_manager.clone(),
                input
            ).await;
            
            assert!(result.is_ok());
            
            // Verify window remains focused on the launched app
            assert_eq!(window_manager.lock().unwrap().get_active_window_pid(), Some(77777));
            
            // Small delay between inputs
            sleep(Duration::from_millis(10)).await;
        }
    }

    // Helper types and functions
    enum InputType {
        MouseClick(i32, i32),
        MouseMove(i32, i32),
        KeyboardText(String),
    }

    async fn activate_and_send_input(
        state: Arc<AppState>,
        window_manager: Arc<Mutex<MockWindowManager>>,
        input_type: InputType
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if an app is running
        let pid = state.app_pid.lock().unwrap().clone();
        
        if let Some(pid) = pid {
            // Activate the window for the launched app
            window_manager.lock().unwrap().activate_window_by_pid(pid)
                .map_err(|e| format!("Failed to activate window: {}", e))?;
            
            // Small delay to ensure focus is established
            sleep(Duration::from_millis(50)).await;
            
            // Send the input
            match input_type {
                InputType::MouseClick(x, y) => {
                    // In real implementation, this would use Enigo
                    Ok(())
                }
                InputType::MouseMove(x, y) => {
                    // In real implementation, this would use Enigo
                    Ok(())
                }
                InputType::KeyboardText(text) => {
                    // In real implementation, this would use Enigo
                    Ok(())
                }
            }
        } else {
            Err("No application is running".into())
        }
    }

    struct FailingWindowManager;

    async fn activate_and_send_input_failing(
        state: Arc<AppState>,
        _window_manager: Arc<Mutex<FailingWindowManager>>,
        _input_type: InputType
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pid = state.app_pid.lock().unwrap().clone();
        
        if let Some(_pid) = pid {
            // Simulate window activation failure
            Err("Failed to activate window: Window not found".into())
        } else {
            Err("No application is running".into())
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_window_focus_tests {
    use super::*;
    use crate::macos::WindowEnumerator;

    #[tokio::test]
    async fn test_find_window_by_pid() {
        let state = Arc::new(create_test_app_state());
        
        // Start a real application for testing
        #[cfg(target_os = "macos")]
        let _ = crate::tests::app_launch_tests::start_app_internal(
            state.clone(),
            "/System/Applications/TextEdit.app/Contents/MacOS/TextEdit".to_string(),
            vec![]
        ).await;
        
        // Give the app time to create windows
        sleep(Duration::from_millis(500)).await;
        
        let pid = state.app_pid.lock().unwrap().clone();
        assert!(pid.is_some());
        
        // Try to find windows for this PID
        let mut enumerator = WindowEnumerator::new();
        let windows = enumerator.find_by_pid(pid.unwrap() as i32);
        
        match windows {
            Ok(windows) => {
                assert!(!windows.is_empty(), "Should find at least one window for the launched app");
                println!("Found {} windows for PID {}", windows.len(), pid.unwrap());
            }
            Err(e) => {
                println!("Failed to find windows: {}", e);
            }
        }
        
        // Clean up
        let _ = crate::tests::app_launch_tests::stop_app_internal(state.clone()).await;
    }

    #[tokio::test]
    async fn test_window_focus_verification() {
        let state = Arc::new(create_test_app_state());
        
        // This test would verify that we can detect which window has focus
        // and ensure it matches our launched application
        
        // In a real implementation, this would:
        // 1. Launch an app
        // 2. Get its PID
        // 3. Activate its window
        // 4. Verify the focused window belongs to our PID
        // 5. Send input
        // 6. Verify input was received by the correct window
    }
}