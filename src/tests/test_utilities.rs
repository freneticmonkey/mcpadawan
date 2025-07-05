#[cfg(test)]
pub mod test_utilities {
    use std::process::{Command, Stdio, Child};
    use std::io::{Write, BufRead, BufReader};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;
    use std::fs::{self, File};
    use std::path::PathBuf;

    /// Test application that can receive and verify input
    pub struct TestInputReceiver {
        process: Child,
        output_file: PathBuf,
        input_log: Arc<Mutex<Vec<String>>>,
    }

    impl TestInputReceiver {
        /// Create a new test input receiver application
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let script = Self::create_receiver_script()?;
            let script_path = script.path().to_path_buf();
            let output_file = script_path.with_extension("output");
            
            // Start the receiver process
            let process = Command::new(&script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            
            // Keep the temp file alive
            std::mem::forget(script);
            
            Ok(Self {
                process,
                output_file,
                input_log: Arc::new(Mutex::new(Vec::new())),
            })
        }

        /// Get the PID of the test application
        pub fn pid(&self) -> u32 {
            self.process.id()
        }

        /// Send text input to the application
        pub fn send_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
            if let Some(stdin) = self.process.stdin.as_mut() {
                writeln!(stdin, "{}", text)?;
                stdin.flush()?;
                self.input_log.lock().unwrap().push(format!("Text: {}", text));
            }
            Ok(())
        }

        /// Simulate receiving a mouse click
        pub fn simulate_mouse_click(&mut self, x: i32, y: i32) -> Result<(), Box<dyn std::error::Error>> {
            self.send_text(&format!("MOUSE_CLICK {} {}", x, y))?;
            self.input_log.lock().unwrap().push(format!("Mouse: click at {},{}", x, y));
            Ok(())
        }

        /// Simulate receiving gamepad input
        pub fn simulate_gamepad(&mut self, button_a: bool, stick_x: i16) -> Result<(), Box<dyn std::error::Error>> {
            self.send_text(&format!("GAMEPAD {} {}", button_a, stick_x))?;
            self.input_log.lock().unwrap().push(format!("Gamepad: A={} X={}", button_a, stick_x));
            Ok(())
        }

        /// Verify that specific input was received
        pub fn verify_received(&self, expected: &str) -> bool {
            if let Ok(content) = fs::read_to_string(&self.output_file) {
                content.contains(expected)
            } else {
                false
            }
        }

        /// Get all received inputs
        pub fn get_received_inputs(&self) -> Vec<String> {
            if let Ok(content) = fs::read_to_string(&self.output_file) {
                content.lines()
                    .filter(|line| !line.is_empty())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                vec![]
            }
        }

        /// Check if the application is still running
        pub fn is_running(&mut self) -> bool {
            match self.process.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        }

        /// Stop the test application
        pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.process.kill()?;
            self.process.wait()?;
            Ok(())
        }

        fn create_receiver_script() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
            let mut file = NamedTempFile::new()?;
            
            #[cfg(target_os = "macos")]
            let script = r#"#!/bin/bash
OUTPUT="${0}.output"
echo "Started PID $$" > "$OUTPUT"

while IFS= read -r line; do
    echo "$(date +%s): $line" >> "$OUTPUT"
    
    # Parse special commands
    case "$line" in
        MOUSE_CLICK*)
            coords="${line#MOUSE_CLICK }"
            echo "Mouse click received at: $coords" >> "$OUTPUT"
            ;;
        GAMEPAD*)
            data="${line#GAMEPAD }"
            echo "Gamepad input received: $data" >> "$OUTPUT"
            ;;
        *)
            echo "Text input: $line" >> "$OUTPUT"
            ;;
    esac
done
"#;

            #[cfg(target_os = "windows")]
            let script = r#"@echo off
set OUTPUT=%~0.output
echo Started PID %RANDOM% > "%OUTPUT%"

:loop
set /p line=
echo %TIME%: %line% >> "%OUTPUT%"

echo %line% | findstr /B "MOUSE_CLICK" >nul
if %errorlevel%==0 (
    echo Mouse click received >> "%OUTPUT%"
)

echo %line% | findstr /B "GAMEPAD" >nul
if %errorlevel%==0 (
    echo Gamepad input received >> "%OUTPUT%"
)

goto loop
"#;

            file.write_all(script.as_bytes())?;
            file.flush()?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = file.as_file().metadata()?.permissions();
                perms.set_mode(0o755);
                file.as_file().set_permissions(perms)?;
            }
            
            Ok(file)
        }
    }

    impl Drop for TestInputReceiver {
        fn drop(&mut self) {
            let _ = self.stop();
            let _ = fs::remove_file(&self.output_file);
        }
    }

    /// Window focus verifier for testing
    pub struct WindowFocusVerifier {
        focused_pid: Arc<Mutex<Option<u32>>>,
    }

    impl WindowFocusVerifier {
        pub fn new() -> Self {
            Self {
                focused_pid: Arc::new(Mutex::new(None)),
            }
        }

        /// Set the currently focused window's PID
        pub fn set_focused_pid(&self, pid: u32) {
            *self.focused_pid.lock().unwrap() = Some(pid);
        }

        /// Check if a specific PID has focus
        pub fn has_focus(&self, pid: u32) -> bool {
            self.focused_pid.lock().unwrap()
                .map(|focused| focused == pid)
                .unwrap_or(false)
        }

        /// Get the currently focused PID
        pub fn get_focused_pid(&self) -> Option<u32> {
            *self.focused_pid.lock().unwrap()
        }

        /// Simulate focusing a window by PID
        pub fn focus_window(&self, pid: u32) -> Result<(), String> {
            // In a real implementation, this would use platform APIs
            self.set_focused_pid(pid);
            Ok(())
        }
    }

    /// Test helper for verifying input targeting
    pub struct InputTargetVerifier {
        receiver: TestInputReceiver,
        focus_verifier: WindowFocusVerifier,
    }

    impl InputTargetVerifier {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let receiver = TestInputReceiver::new()?;
            let focus_verifier = WindowFocusVerifier::new();
            
            Ok(Self {
                receiver,
                focus_verifier,
            })
        }

        /// Verify that input is only sent when the app has focus
        pub fn verify_focused_input(&mut self, text: &str) -> Result<bool, Box<dyn std::error::Error>> {
            let pid = self.receiver.pid();
            
            // First, try sending without focus
            self.focus_verifier.set_focused_pid(99999); // Different PID
            thread::sleep(Duration::from_millis(50));
            
            // This should not be received
            self.receiver.send_text(&format!("UNFOCUSED: {}", text))?;
            thread::sleep(Duration::from_millis(100));
            
            // Now focus the correct window
            self.focus_verifier.focus_window(pid)?;
            thread::sleep(Duration::from_millis(50));
            
            // This should be received
            self.receiver.send_text(&format!("FOCUSED: {}", text))?;
            thread::sleep(Duration::from_millis(100));
            
            // Verify only the focused input was received
            let received = self.receiver.get_received_inputs();
            let has_focused = received.iter().any(|s| s.contains(&format!("FOCUSED: {}", text)));
            let has_unfocused = received.iter().any(|s| s.contains(&format!("UNFOCUSED: {}", text)));
            
            Ok(has_focused && !has_unfocused)
        }

        pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.receiver.stop()
        }
    }

    /// Create a mock GUI application for testing
    #[cfg(target_os = "macos")]
    pub fn create_mock_gui_app() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        
        let script = r#"#!/usr/bin/osascript
on run
    set outputFile to (POSIX path of (path to me)) & ".output"
    
    tell application "System Events"
        set frontApp to name of first application process whose frontmost is true
        do shell script "echo 'Initial front app: " & frontApp & "' > " & quoted form of outputFile
    end tell
    
    repeat
        delay 0.1
        tell application "System Events"
            if frontmost of application process "Terminal" then
                try
                    set theKeys to (keystroke)
                    do shell script "echo 'Received: " & theKeys & "' >> " & quoted form of outputFile
                end try
            end if
        end tell
    end repeat
end run
"#;

        file.write_all(script.as_bytes())?;
        file.flush()?;
        
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.as_file().metadata()?.permissions();
        perms.set_mode(0o755);
        file.as_file().set_permissions(perms)?;
        
        Ok(file)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_input_receiver_creation() {
            let receiver = TestInputReceiver::new();
            assert!(receiver.is_ok());
            
            if let Ok(mut receiver) = receiver {
                assert!(receiver.pid() > 0);
                assert!(receiver.is_running());
                let _ = receiver.stop();
            }
        }

        #[test]
        fn test_window_focus_verifier() {
            let verifier = WindowFocusVerifier::new();
            
            assert_eq!(verifier.get_focused_pid(), None);
            
            verifier.set_focused_pid(12345);
            assert!(verifier.has_focus(12345));
            assert!(!verifier.has_focus(54321));
            
            let _ = verifier.focus_window(54321);
            assert!(verifier.has_focus(54321));
            assert!(!verifier.has_focus(12345));
        }

        #[test]
        fn test_input_logging() {
            if let Ok(mut receiver) = TestInputReceiver::new() {
                let _ = receiver.send_text("Test input");
                let _ = receiver.simulate_mouse_click(100, 200);
                let _ = receiver.simulate_gamepad(true, 1000);
                
                thread::sleep(Duration::from_millis(200));
                
                let inputs = receiver.get_received_inputs();
                assert!(!inputs.is_empty());
                
                let _ = receiver.stop();
            }
        }
    }
}