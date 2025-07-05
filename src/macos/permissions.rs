use super::types::PermissionStatus;
use crate::error::CaptureError;

#[cfg(target_os = "macos")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn check_screen_recording_permission() -> PermissionStatus {
        #[cfg(target_os = "macos")]
        {
            unsafe {
                let trusted = CGPreflightScreenCaptureAccess();
                if trusted {
                    PermissionStatus::Granted
                } else {
                    PermissionStatus::NotDetermined
                }
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            PermissionStatus::NotApplicable
        }
    }

    pub fn request_screen_recording_permission() -> PermissionStatus {
        #[cfg(target_os = "macos")]
        {
            unsafe {
                let granted = CGRequestScreenCaptureAccess();
                if granted {
                    PermissionStatus::Granted
                } else {
                    PermissionStatus::Denied
                }
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            PermissionStatus::NotApplicable
        }
    }

    pub fn ensure_permission() -> Result<(), CaptureError> {
        let status = Self::check_screen_recording_permission();
        
        match status {
            PermissionStatus::Granted => Ok(()),
            PermissionStatus::NotDetermined => {
                let new_status = Self::request_screen_recording_permission();
                match new_status {
                    PermissionStatus::Granted => Ok(()),
                    _ => Err(Self::create_permission_error()),
                }
            }
            PermissionStatus::Denied => Err(Self::create_permission_error()),
            PermissionStatus::NotApplicable => Ok(()),
        }
    }

    pub fn create_permission_error() -> CaptureError {
        CaptureError::PermissionDenied {
            message: "Screen Recording permission is required to capture windows. \
                     Please enable Screen Recording permission in System Preferences > \
                     Security & Privacy > Privacy > Screen Recording.".to_string(),
            help_url: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture".to_string(),
        }
    }

    pub fn open_system_preferences() {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                .spawn()
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_error_creation() {
        let error = PermissionManager::create_permission_error();
        match error {
            CaptureError::PermissionDenied { message, help_url } => {
                assert!(message.contains("Screen Recording permission"));
                assert!(help_url.contains("systempreferences"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }
}