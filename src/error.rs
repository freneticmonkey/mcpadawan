use core_graphics::display::CGWindowID;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MacOSError {
    #[error("Failed to enumerate windows")]
    WindowListError,
    
    #[error("Window not found: {0}")]
    WindowNotFound(CGWindowID),
    
    #[error("Capture failed for window {window_id:?}: {message}")]
    CaptureError {
        window_id: Option<CGWindowID>,
        message: String,
    },
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Core Foundation error: {0}")]
    CoreFoundationError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("No suitable capture strategy available")]
    NoSuitableStrategy,
    
    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),
    
    #[error("Target not supported by this strategy")]
    UnsupportedTarget,
    
    #[error("Permission denied: {message}")]
    PermissionDenied {
        message: String,
        help_url: String,
    },
    
    #[error("Window destroyed during capture")]
    WindowDestroyed,
    
    #[error("Fallback capture failed: {0}")]
    FallbackError(String),
    
    #[error("Platform error: {0}")]
    PlatformError(#[from] MacOSError),
    
    #[error("Conversion error: {0}")]
    ConversionError(#[from] ConversionError),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Invalid image dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    
    #[error("No data provider available")]
    NoDataProvider,
    
    #[error("Unsupported pixel format: {bits_per_pixel} bits per pixel, {bits_per_component} bits per component")]
    UnsupportedFormat {
        bits_per_pixel: usize,
        bits_per_component: usize,
    },
    
    #[error("Buffer too small for image data")]
    BufferTooSmall,
    
    #[error("Memory allocation failed")]
    MemoryAllocationFailed,
    
    #[error("Color space conversion failed")]
    ColorSpaceConversionFailed,
}

impl CaptureError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            CaptureError::NoSuitableStrategy => false,
            CaptureError::StrategyNotFound(_) => false,
            CaptureError::UnsupportedTarget => false,
            CaptureError::PermissionDenied { .. } => false,
            CaptureError::WindowDestroyed => true,
            CaptureError::FallbackError(_) => true,
            CaptureError::PlatformError(e) => e.is_recoverable(),
            CaptureError::ConversionError(_) => false,
            CaptureError::IoError(_) => false,
            CaptureError::ImageError(_) => false,
        }
    }
}

impl MacOSError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            MacOSError::WindowListError => true,
            MacOSError::WindowNotFound(_) => true,
            MacOSError::CaptureError { .. } => true,
            MacOSError::PermissionDenied(_) => false,
            MacOSError::CoreFoundationError(_) => false,
            MacOSError::InvalidInput(_) => false,
            MacOSError::SystemError(_) => false,
        }
    }
}

impl ConversionError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            ConversionError::InvalidDimensions { .. } => false,
            ConversionError::NoDataProvider => false,
            ConversionError::UnsupportedFormat { .. } => false,
            ConversionError::BufferTooSmall => false,
            ConversionError::MemoryAllocationFailed => true,
            ConversionError::ColorSpaceConversionFailed => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        assert!(!CaptureError::PermissionDenied {
            message: "test".to_string(),
            help_url: "test".to_string(),
        }.is_recoverable());
        
        assert!(CaptureError::WindowDestroyed.is_recoverable());
        assert!(MacOSError::WindowListError.is_recoverable());
        assert!(!MacOSError::PermissionDenied("test".to_string()).is_recoverable());
    }
}