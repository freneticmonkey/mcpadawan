use super::{CaptureStrategy, CaptureTarget};
use crate::error::CaptureError;
use image::RgbaImage;
use std::process::Command;

pub struct FallbackCapture;

impl FallbackCapture {
    pub fn new() -> Self {
        Self
    }

    fn capture_with_screencapture(&self, target: CaptureTarget) -> Result<RgbaImage, CaptureError> {
        let temp_file = format!("/tmp/mcpadawan_capture_{}.png", std::process::id());
        
        let mut cmd = Command::new("screencapture");
        cmd.arg("-x"); // No sound
        
        match target {
            CaptureTarget::Window { id } => {
                cmd.arg("-l").arg(id.to_string());
            }
            CaptureTarget::Region { bounds } => {
                let rect = format!("{},{},{},{}", 
                    bounds.origin.x as i32,
                    bounds.origin.y as i32,
                    bounds.size.width as i32,
                    bounds.size.height as i32
                );
                cmd.arg("-R").arg(rect);
            }
            CaptureTarget::FullScreen => {
                // Default behavior captures full screen
            }
            CaptureTarget::Windows { .. } => {
                return Err(CaptureError::UnsupportedTarget);
            }
        }
        
        cmd.arg(&temp_file);
        
        let output = cmd.output()
            .map_err(|e| CaptureError::FallbackError(format!("Failed to execute screencapture: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(CaptureError::FallbackError(format!("screencapture failed: {}", error_msg)));
        }
        
        let image = image::open(&temp_file)
            .map_err(|e| CaptureError::FallbackError(format!("Failed to open captured image: {}", e)))?
            .into_rgba8();
        
        std::fs::remove_file(&temp_file).ok();
        
        Ok(image)
    }

    fn is_screencapture_available(&self) -> bool {
        Command::new("which")
            .arg("screencapture")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl CaptureStrategy for FallbackCapture {
    fn capture(&self, target: CaptureTarget) -> Result<RgbaImage, CaptureError> {
        if !self.is_screencapture_available() {
            return Err(CaptureError::FallbackError("screencapture command not available".to_string()));
        }
        
        self.capture_with_screencapture(target)
    }

    fn supports_target(&self, target: &CaptureTarget) -> bool {
        #[cfg(target_os = "macos")]
        {
            match target {
                CaptureTarget::Window { .. } => self.is_screencapture_available(),
                CaptureTarget::Region { .. } => self.is_screencapture_available(),
                CaptureTarget::FullScreen => self.is_screencapture_available(),
                CaptureTarget::Windows { .. } => false, // Not supported by screencapture
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn priority(&self) -> u32 {
        10
    }

    fn name(&self) -> &'static str {
        "fallback-screencapture"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_priority() {
        let capture = FallbackCapture::new();
        assert_eq!(capture.priority(), 10);
    }

    #[test]
    fn test_fallback_name() {
        let capture = FallbackCapture::new();
        assert_eq!(capture.name(), "fallback-screencapture");
    }
}