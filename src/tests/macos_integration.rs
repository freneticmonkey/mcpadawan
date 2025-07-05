#[cfg(target_os = "macos")]
mod macos_tests {
    use crate::macos::{WindowEnumerator, NativeWindowCapture, ImageConverter, PermissionManager};
    use crate::capture::{CaptureManager, CaptureTarget};
    use crate::capture::native::NativeMacOSCapture;
    use std::time::Duration;

    #[test]
    fn test_window_enumeration() {
        let mut enumerator = WindowEnumerator::new();
        let windows = enumerator.enumerate();
        
        match windows {
            Ok(windows) => {
                assert!(!windows.is_empty(), "Should find at least one window");
                
                for window in windows.iter().take(3) {
                    println!("Window: {} (PID: {}) - {}x{}", 
                        window.name.as_ref().unwrap_or(&"Unknown".to_string()),
                        window.owner_pid,
                        window.bounds.size.width as i32,
                        window.bounds.size.height as i32
                    );
                }
            }
            Err(e) => {
                println!("Window enumeration failed: {}", e);
            }
        }
    }

    #[test]
    fn test_permission_check() {
        let status = PermissionManager::check_screen_recording_permission();
        println!("Screen recording permission status: {:?}", status);
        
        // This test just checks that the permission check doesn't crash
        // The actual permission status depends on system configuration
    }

    #[test]
    fn test_capture_manager_setup() {
        let mut manager = CaptureManager::new();
        manager.add_strategy(Box::new(NativeMacOSCapture::new()));
        
        let strategies = manager.get_available_strategies();
        assert!(strategies.contains(&"native-macos"));
        
        // Test that the strategy supports full screen capture
        let native_capture = NativeMacOSCapture::new();
        assert!(native_capture.supports_target(&CaptureTarget::FullScreen));
    }

    #[test]
    fn test_window_enumeration_with_cache() {
        let mut enumerator = WindowEnumerator::new()
            .with_cache(Duration::from_secs(1));
        
        let first_call = enumerator.enumerate();
        let second_call = enumerator.enumerate();
        
        match (first_call, second_call) {
            (Ok(first), Ok(second)) => {
                // Both calls should succeed
                assert!(!first.is_empty());
                assert!(!second.is_empty());
                
                println!("Found {} windows in first call, {} in second call", 
                    first.len(), second.len());
            }
            (Err(e), _) | (_, Err(e)) => {
                println!("Window enumeration failed: {}", e);
            }
        }
    }

    #[test]
    fn test_find_windows_by_name() {
        let mut enumerator = WindowEnumerator::new();
        
        let finder_windows = enumerator.find_by_name("Finder");
        match finder_windows {
            Ok(windows) => {
                println!("Found {} Finder windows", windows.len());
                for window in windows {
                    println!("  - {}", window.name.as_ref().unwrap_or(&"Unknown".to_string()));
                }
            }
            Err(e) => {
                println!("Failed to find Finder windows: {}", e);
            }
        }
    }

    #[test]
    fn test_image_converter_creation() {
        let converter = ImageConverter::new();
        // This test just ensures the ImageConverter can be created without panicking
        println!("ImageConverter created successfully");
    }
}

#[cfg(not(target_os = "macos"))]
mod non_macos_tests {
    #[test]
    fn test_non_macos_compilation() {
        // This test ensures the code compiles on non-macOS platforms
        println!("Running on non-macOS platform");
    }
}