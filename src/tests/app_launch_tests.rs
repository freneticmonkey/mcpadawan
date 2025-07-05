#[cfg(test)]
mod app_launch_tests {
    use crate::{AppState, ThreadSafeEnigo};
    use std::sync::{Arc, Mutex};
    use std::process::{Command, Stdio, Child};
    use std::time::Duration;
    use tokio::time::sleep;

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

    #[test]
    fn test_app_state_initialization() {
        let state = create_test_app_state();
        assert!(state.app_process.lock().unwrap().is_none());
        assert!(state.app_pid.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_start_app_success() {
        let state = Arc::new(create_test_app_state());
        
        // Start a simple command that will stay alive
        #[cfg(target_os = "macos")]
        let result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/yes".to_string(),
            vec![]
        ).await;
        
        #[cfg(target_os = "windows")]
        let result = crate::start_app_internal(
            state.clone(),
            "cmd.exe".to_string(),
            vec!["/c".to_string(), "timeout".to_string(), "/t".to_string(), "10".to_string()]
        ).await;
        
        assert!(result.is_ok());
        
        // Verify state was updated
        assert!(state.app_process.lock().unwrap().is_some());
        assert!(state.app_pid.lock().unwrap().is_some());
        
        // Clean up
        let _ = crate::stop_app_internal(state.clone()).await;
    }

    #[tokio::test]
    async fn test_start_app_invalid_executable() {
        let state = Arc::new(create_test_app_state());
        
        let result = crate::start_app_internal(
            state.clone(),
            "/nonexistent/app".to_string(),
            vec![]
        ).await;
        
        assert!(result.is_err());
        assert!(state.app_process.lock().unwrap().is_none());
        assert!(state.app_pid.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_stop_app_when_running() {
        let state = Arc::new(create_test_app_state());
        
        // Start an app first
        #[cfg(target_os = "macos")]
        let start_result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/yes".to_string(),
            vec![]
        ).await;
        
        #[cfg(target_os = "windows")]
        let start_result = crate::start_app_internal(
            state.clone(),
            "cmd.exe".to_string(),
            vec!["/c".to_string(), "timeout".to_string(), "/t".to_string(), "10".to_string()]
        ).await;
        
        assert!(start_result.is_ok());
        
        // Stop the app
        let stop_result = crate::stop_app_internal(state.clone()).await;
        assert!(stop_result.is_ok());
        
        // Verify state was cleared
        assert!(state.app_process.lock().unwrap().is_none());
        assert!(state.app_pid.lock().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_stop_app_when_not_running() {
        let state = Arc::new(create_test_app_state());
        
        let result = crate::stop_app_internal(state.clone()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_app_twice_fails() {
        let state = Arc::new(create_test_app_state());
        
        // Start first app
        #[cfg(target_os = "macos")]
        let first_result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/yes".to_string(),
            vec![]
        ).await;
        
        #[cfg(target_os = "windows")]
        let first_result = crate::start_app_internal(
            state.clone(),
            "cmd.exe".to_string(),
            vec!["/c".to_string(), "timeout".to_string(), "/t".to_string(), "10".to_string()]
        ).await;
        
        assert!(first_result.is_ok());
        
        // Try to start second app without stopping first
        let second_result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/echo".to_string(),
            vec!["test".to_string()]
        ).await;
        
        assert!(second_result.is_err());
        
        // Clean up
        let _ = crate::stop_app_internal(state.clone()).await;
    }

    #[tokio::test]
    async fn test_app_pid_tracking() {
        let state = Arc::new(create_test_app_state());
        
        #[cfg(target_os = "macos")]
        let result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/yes".to_string(),
            vec![]
        ).await;
        
        #[cfg(target_os = "windows")]
        let result = crate::start_app_internal(
            state.clone(),
            "cmd.exe".to_string(),
            vec!["/c".to_string(), "timeout".to_string(), "/t".to_string(), "10".to_string()]
        ).await;
        
        assert!(result.is_ok());
        
        let pid = state.app_pid.lock().unwrap().clone();
        assert!(pid.is_some());
        assert!(pid.unwrap() > 0);
        
        // Clean up
        let _ = crate::stop_app_internal(state.clone()).await;
    }

    #[tokio::test] 
    async fn test_app_with_arguments() {
        let state = Arc::new(create_test_app_state());
        
        #[cfg(target_os = "macos")]
        let result = crate::start_app_internal(
            state.clone(),
            "/usr/bin/echo".to_string(),
            vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]
        ).await;
        
        #[cfg(target_os = "windows")]
        let result = crate::start_app_internal(
            state.clone(),
            "cmd.exe".to_string(),
            vec!["/c".to_string(), "echo".to_string(), "arg1".to_string(), "arg2".to_string()]
        ).await;
        
        assert!(result.is_ok());
        
        // Give the process time to complete
        sleep(Duration::from_millis(100)).await;
        
        // For echo, the process should complete quickly
        // Clean up if still running
        let _ = crate::stop_app_internal(state.clone()).await;
    }
}

// Helper functions that would be implemented in lib.rs or mcp.rs
#[cfg(test)]
pub async fn start_app_internal(
    state: Arc<AppState>,
    exe_path: String,
    args: Vec<String>
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if an app is already running
    if state.app_process.lock().unwrap().is_some() {
        return Err("An application is already running".into());
    }

    // Start the application
    let mut cmd = Command::new(&exe_path);
    cmd.args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            *state.app_process.lock().unwrap() = Some(child);
            *state.app_pid.lock().unwrap() = Some(pid);
            Ok(())
        }
        Err(e) => Err(format!("Failed to start application: {}", e).into())
    }
}

#[cfg(test)]
pub async fn stop_app_internal(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut process_guard = state.app_process.lock().unwrap();
    
    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                *state.app_pid.lock().unwrap() = None;
                Ok(())
            }
            Err(e) => Err(format!("Failed to stop application: {}", e).into())
        }
    } else {
        Err("No application is running".into())
    }
}