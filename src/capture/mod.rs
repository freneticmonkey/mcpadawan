use crate::error::CaptureError;
use core_graphics::display::CGWindowID;
use core_graphics::geometry::CGRect;
use image::RgbaImage;

#[cfg(target_os = "macos")]
pub mod native;
#[cfg(target_os = "macos")]
pub mod fallback;

pub trait CaptureStrategy: Send + Sync {
    fn capture(&self, target: CaptureTarget) -> Result<RgbaImage, CaptureError>;
    fn supports_target(&self, target: &CaptureTarget) -> bool;
    fn priority(&self) -> u32;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    Window { id: CGWindowID },
    Region { bounds: CGRect },
    FullScreen,
    Windows { ids: Vec<CGWindowID> },
}

pub struct CaptureManager {
    strategies: Vec<Box<dyn CaptureStrategy>>,
}

impl CaptureManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn CaptureStrategy>) {
        self.strategies.push(strategy);
        self.strategies.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    pub fn capture(&self, target: CaptureTarget) -> Result<RgbaImage, CaptureError> {
        let mut last_error = None;
        
        for strategy in &self.strategies {
            if !strategy.supports_target(&target) {
                continue;
            }

            match strategy.capture(target.clone()) {
                Ok(image) => return Ok(image),
                Err(e) => {
                    eprintln!("Strategy '{}' failed: {}", strategy.name(), e);
                    if !e.is_recoverable() {
                        return Err(e);
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(CaptureError::NoSuitableStrategy))
    }

    pub fn get_available_strategies(&self) -> Vec<&str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }

    pub fn capture_with_strategy(&self, target: CaptureTarget, strategy_name: &str) -> Result<RgbaImage, CaptureError> {
        let strategy = self.strategies.iter()
            .find(|s| s.name() == strategy_name)
            .ok_or(CaptureError::StrategyNotFound(strategy_name.to_string()))?;

        if !strategy.supports_target(&target) {
            return Err(CaptureError::UnsupportedTarget);
        }

        strategy.capture(target)
    }
}

impl Default for CaptureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStrategy {
        name: &'static str,
        priority: u32,
        should_fail: bool,
    }

    impl CaptureStrategy for MockStrategy {
        fn capture(&self, _target: CaptureTarget) -> Result<RgbaImage, CaptureError> {
            if self.should_fail {
                Err(CaptureError::NoSuitableStrategy)
            } else {
                Ok(RgbaImage::new(1, 1))
            }
        }

        fn supports_target(&self, _target: &CaptureTarget) -> bool {
            true
        }

        fn priority(&self) -> u32 {
            self.priority
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn test_strategy_priority_ordering() {
        let mut manager = CaptureManager::new();
        
        manager.add_strategy(Box::new(MockStrategy {
            name: "low",
            priority: 1,
            should_fail: false,
        }));
        
        manager.add_strategy(Box::new(MockStrategy {
            name: "high",
            priority: 10,
            should_fail: false,
        }));

        let strategies = manager.get_available_strategies();
        assert_eq!(strategies[0], "high");
        assert_eq!(strategies[1], "low");
    }
}