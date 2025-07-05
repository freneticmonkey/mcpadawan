use super::types::CaptureOptions;
use crate::error::MacOSError;
use core_graphics::display::{CGWindowID, CGWindowListOption, kCGNullWindowID, kCGWindowListOptionAll, kCGWindowListOptionOnScreenOnly};
use core_graphics::geometry::CGRect;
use core_graphics::image::CGImage;

extern "C" {
    fn CGWindowListCreateImage(
        screenBounds: CGRect,
        listOption: CGWindowListOption,
        windowID: CGWindowID,
        imageOption: u32,
    ) -> *mut CGImage;

    fn CGWindowListCreateImageFromArray(
        screenBounds: CGRect,
        windowArray: *const CGWindowID,
        arrayCount: usize,
        imageOption: u32,
    ) -> *mut CGImage;
}

const kCGWindowImageDefault: u32 = 0;
const kCGWindowImageBoundsIgnoreFraming: u32 = 1 << 0;
const kCGWindowImageShouldBeOpaque: u32 = 1 << 1;
const kCGWindowImageOnlyShadows: u32 = 1 << 2;

pub struct NativeWindowCapture {
    options: CaptureOptions,
}

impl NativeWindowCapture {
    pub fn new() -> Self {
        Self {
            options: CaptureOptions::default(),
        }
    }

    pub fn with_options(options: CaptureOptions) -> Self {
        Self { options }
    }

    pub fn capture_window(&self, window_id: CGWindowID) -> Result<*mut CGImage, MacOSError> {
        unsafe {
            let window_array = [window_id];
            let image_option = self.build_image_options();
            
            let cg_image = CGWindowListCreateImageFromArray(
                CGRect::new(
                    &core_graphics::geometry::CGPoint::new(0.0, 0.0),
                    &core_graphics::geometry::CGSize::new(0.0, 0.0)
                ),
                window_array.as_ptr(),
                window_array.len(),
                image_option,
            );

            if cg_image.is_null() {
                return Err(MacOSError::CaptureError {
                    window_id: Some(window_id),
                    message: "Failed to capture window".to_string(),
                });
            }

            Ok(cg_image)
        }
    }

    pub fn capture_region(&self, bounds: CGRect) -> Result<*mut CGImage, MacOSError> {
        unsafe {
            let list_option = if self.options.only_onscreen {
                kCGWindowListOptionOnScreenOnly
            } else {
                kCGWindowListOptionAll
            };
            
            let image_option = self.build_image_options();

            let cg_image = CGWindowListCreateImage(
                bounds,
                list_option,
                kCGNullWindowID,
                image_option,
            );

            if cg_image.is_null() {
                return Err(MacOSError::CaptureError {
                    window_id: None,
                    message: "Failed to capture region".to_string(),
                });
            }

            Ok(cg_image)
        }
    }

    pub fn capture_fullscreen(&self) -> Result<*mut CGImage, MacOSError> {
        self.capture_region(CGRect::new(
            &core_graphics::geometry::CGPoint::new(0.0, 0.0),
            &core_graphics::geometry::CGSize::new(0.0, 0.0)
        ))
    }

    pub fn capture_windows(&self, window_ids: &[CGWindowID]) -> Result<*mut CGImage, MacOSError> {
        if window_ids.is_empty() {
            return Err(MacOSError::InvalidInput("No window IDs provided".to_string()));
        }

        unsafe {
            let image_option = self.build_image_options();
            
            let cg_image = CGWindowListCreateImageFromArray(
                CGRect::new(
                    &core_graphics::geometry::CGPoint::new(0.0, 0.0),
                    &core_graphics::geometry::CGSize::new(0.0, 0.0)
                ),
                window_ids.as_ptr(),
                window_ids.len(),
                image_option,
            );

            if cg_image.is_null() {
                return Err(MacOSError::CaptureError {
                    window_id: None,
                    message: format!("Failed to capture {} windows", window_ids.len()),
                });
            }

            Ok(cg_image)
        }
    }

    fn build_image_options(&self) -> u32 {
        let mut options = kCGWindowImageDefault;
        
        if !self.options.include_shadows {
            options |= kCGWindowImageBoundsIgnoreFraming;
        }
        
        if self.options.fill_background {
            options |= kCGWindowImageShouldBeOpaque;
        }
        
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_options() {
        let capture = NativeWindowCapture::new();
        assert_eq!(capture.build_image_options(), kCGWindowImageShouldBeOpaque);

        let capture_with_shadows = NativeWindowCapture::with_options(CaptureOptions {
            include_shadows: true,
            only_onscreen: true,
            fill_background: false,
        });
        assert_eq!(capture_with_shadows.build_image_options(), kCGWindowImageDefault);
    }
}