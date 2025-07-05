use super::{CaptureStrategy, CaptureTarget};
use crate::error::CaptureError;
use crate::macos::{NativeWindowCapture, ImageConverter, PermissionManager, CaptureOptions};
use image::RgbaImage;

pub struct NativeMacOSCapture {
    capture: NativeWindowCapture,
    converter: ImageConverter,
}

impl NativeMacOSCapture {
    pub fn new() -> Self {
        Self {
            capture: NativeWindowCapture::new(),
            converter: ImageConverter::new(),
        }
    }

    pub fn with_options(options: CaptureOptions) -> Self {
        Self {
            capture: NativeWindowCapture::with_options(options),
            converter: ImageConverter::new(),
        }
    }
}

impl CaptureStrategy for NativeMacOSCapture {
    fn capture(&self, target: CaptureTarget) -> Result<RgbaImage, CaptureError> {
        PermissionManager::ensure_permission()?;

        let cg_image = match target {
            CaptureTarget::Window { id } => {
                self.capture.capture_window(id)
                    .map_err(CaptureError::PlatformError)?
            }
            CaptureTarget::Region { bounds } => {
                self.capture.capture_region(bounds)
                    .map_err(CaptureError::PlatformError)?
            }
            CaptureTarget::FullScreen => {
                self.capture.capture_fullscreen()
                    .map_err(CaptureError::PlatformError)?
            }
            CaptureTarget::Windows { ids } => {
                self.capture.capture_windows(&ids)
                    .map_err(CaptureError::PlatformError)?
            }
        };

        let mut converter = ImageConverter::new();
        converter.cgimage_to_rgba(cg_image)
            .map_err(CaptureError::ConversionError)
    }

    fn supports_target(&self, target: &CaptureTarget) -> bool {
        #[cfg(target_os = "macos")]
        {
            match target {
                CaptureTarget::Window { .. } => true,
                CaptureTarget::Region { .. } => true,
                CaptureTarget::FullScreen => true,
                CaptureTarget::Windows { ids } => !ids.is_empty(),
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn priority(&self) -> u32 {
        100
    }

    fn name(&self) -> &'static str {
        "native-macos"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_graphics::display::CGWindowID;

    #[test]
    fn test_supports_target() {
        let capture = NativeMacOSCapture::new();
        
        assert!(capture.supports_target(&CaptureTarget::Window { id: 123 }));
        assert!(capture.supports_target(&CaptureTarget::FullScreen));
        assert!(capture.supports_target(&CaptureTarget::Windows { ids: vec![123] }));
        assert!(!capture.supports_target(&CaptureTarget::Windows { ids: vec![] }));
    }

    #[test]
    fn test_priority() {
        let capture = NativeMacOSCapture::new();
        assert_eq!(capture.priority(), 100);
    }

    #[test]
    fn test_name() {
        let capture = NativeMacOSCapture::new();
        assert_eq!(capture.name(), "native-macos");
    }
}