use core_foundation::base::{CFRelease, CFTypeRef};
use core_graphics::display::CGWindowID;
use core_graphics::geometry::CGRect;
use std::fmt;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: CGWindowID,
    pub name: Option<String>,
    pub owner_name: Option<String>,
    pub owner_pid: u32,
    pub bounds: CGRect,
    pub layer: i32,
    pub alpha: f32,
    pub is_onscreen: bool,
}

impl WindowInfo {
    pub fn is_normal_window(&self) -> bool {
        self.layer == 0 && 
        self.alpha > 0.0 && 
        self.is_onscreen &&
        self.bounds.size.width > 0.0 &&
        self.bounds.size.height > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    BGRA,
    RGBA,
    RGB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    NotApplicable,
}

pub struct CFObject<T> {
    ptr: *mut T,
}

impl<T> CFObject<T> {
    pub unsafe fn from_ptr(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }
}

impl<T> Drop for CFObject<T> {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.ptr as CFTypeRef);
        }
    }
}

impl<T> fmt::Debug for CFObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CFObject")
            .field("ptr", &self.ptr)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct CaptureOptions {
    pub include_shadows: bool,
    pub only_onscreen: bool,
    pub fill_background: bool,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            include_shadows: false,
            only_onscreen: true,
            fill_background: true,
        }
    }
}