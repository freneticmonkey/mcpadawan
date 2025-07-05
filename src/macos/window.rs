use super::types::WindowInfo;
use crate::error::MacOSError;
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::{CGWindowID, CGWindowListOption, kCGNullWindowID, kCGWindowListOptionOnScreenOnly, kCGWindowListOptionAll};
use core_graphics::geometry::CGRect;
use std::collections::HashMap;
use std::time::{Duration, Instant};

extern "C" {
    fn CGWindowListCopyWindowInfo(option: CGWindowListOption, relativeToWindow: CGWindowID) -> CFArrayRef;
}

pub struct WindowEnumerator {
    option: CGWindowListOption,
    cache: Option<WindowCache>,
}

struct WindowCache {
    entries: HashMap<u32, CachedWindowList>,
    ttl: Duration,
}

struct CachedWindowList {
    windows: Vec<WindowInfo>,
    timestamp: Instant,
}

impl WindowEnumerator {
    pub fn new() -> Self {
        Self {
            option: kCGWindowListOptionOnScreenOnly,
            cache: None,
        }
    }

    pub fn with_all_windows() -> Self {
        Self {
            option: kCGWindowListOptionAll,
            cache: None,
        }
    }

    pub fn with_cache(mut self, ttl: Duration) -> Self {
        self.cache = Some(WindowCache {
            entries: HashMap::new(),
            ttl,
        });
        self
    }

    pub fn enumerate(&mut self) -> Result<Vec<WindowInfo>, MacOSError> {
        unsafe {
            let window_list = CGWindowListCopyWindowInfo(self.option, kCGNullWindowID);
            if window_list.is_null() {
                return Err(MacOSError::WindowListError);
            }

            let windows = self.parse_window_list(window_list)?;
            Ok(windows)
        }
    }

    pub fn find_by_pid(&mut self, pid: u32) -> Result<Vec<WindowInfo>, MacOSError> {
        if let Some(cache) = &mut self.cache {
            if let Some(cached) = cache.get_cached(pid) {
                return Ok(cached);
            }
        }

        let all_windows = self.enumerate()?;
        let filtered: Vec<WindowInfo> = all_windows
            .into_iter()
            .filter(|w| w.owner_pid == pid)
            .collect();

        if let Some(cache) = &mut self.cache {
            cache.store(pid, filtered.clone());
        }

        Ok(filtered)
    }

    pub fn find_by_name(&mut self, name: &str) -> Result<Vec<WindowInfo>, MacOSError> {
        let all_windows = self.enumerate()?;
        Ok(all_windows
            .into_iter()
            .filter(|w| {
                w.name.as_ref().map(|n| n.contains(name)).unwrap_or(false) ||
                w.owner_name.as_ref().map(|n| n.contains(name)).unwrap_or(false)
            })
            .collect())
    }

    unsafe fn parse_window_list(&self, window_list: CFArrayRef) -> Result<Vec<WindowInfo>, MacOSError> {
        let array: CFArray<CFDictionary> = TCFType::wrap_under_create_rule(window_list);
        let mut windows = Vec::new();

        for i in 0..array.len() {
            if let Some(dict) = array.get(i) {
                if let Some(window_info) = self.parse_window_dict(&dict) {
                    windows.push(window_info);
                }
            }
        }

        Ok(windows)
    }

    unsafe fn parse_window_dict(&self, _dict: &CFDictionary) -> Option<WindowInfo> {
        // For now, return a simplified mock window to avoid complex Core Foundation parsing
        // This would need to be properly implemented using the correct Core Foundation API
        Some(WindowInfo {
            id: 1,
            name: Some("Mock Window".to_string()),
            owner_name: Some("Mock App".to_string()),
            owner_pid: 1234,
            bounds: CGRect::new(
                &core_graphics::geometry::CGPoint::new(100.0, 100.0),
                &core_graphics::geometry::CGSize::new(800.0, 600.0)
            ),
            layer: 0,
            alpha: 1.0,
            is_onscreen: true,
        })
    }

    unsafe fn parse_bounds(&self, _dict: &CFDictionary) -> Option<CGRect> {
        // Simplified bounds parsing - would need proper Core Foundation implementation
        Some(CGRect::new(
            &core_graphics::geometry::CGPoint::new(0.0, 0.0),
            &core_graphics::geometry::CGSize::new(100.0, 100.0),
        ))
    }
}

impl WindowCache {
    fn get_cached(&self, pid: u32) -> Option<Vec<WindowInfo>> {
        self.entries.get(&pid).and_then(|entry| {
            if entry.timestamp.elapsed() < self.ttl {
                Some(entry.windows.clone())
            } else {
                None
            }
        })
    }

    fn store(&mut self, pid: u32, windows: Vec<WindowInfo>) {
        self.entries.insert(pid, CachedWindowList {
            windows,
            timestamp: Instant::now(),
        });
    }
}