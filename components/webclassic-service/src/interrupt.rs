use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct InterruptSource {
    interrupted: Arc<AtomicBool>,
}

impl InterruptSource {
    pub fn new() -> Self {
        Self {
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn trigger(&self) {
        self.interrupted.store(true, Ordering::Release);
    }

    pub fn reset(&self) {
        self.interrupted.store(false, Ordering::Release);
    }

    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Acquire)
    }

    pub fn subscribe(&self) -> Interrupt {
        Interrupt {
            interrupted: Arc::clone(&self.interrupted),
        }
    }
}

impl Default for InterruptSource {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Interrupt {
    interrupted: Arc<AtomicBool>,
}

impl Interrupt {
    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_lifecycle() {
        let source = InterruptSource::new();
        assert!(!source.is_interrupted());
        source.trigger();
        assert!(source.is_interrupted());
        source.reset();
        assert!(!source.is_interrupted());
    }

    #[test]
    fn subscribers_observe_trigger() {
        let source = InterruptSource::new();
        let sub1 = source.subscribe();
        let sub2 = source.subscribe();
        let sub1_clone = sub1.clone();

        assert!(!sub1.is_interrupted());
        source.trigger();
        assert!(sub1.is_interrupted());
        assert!(sub2.is_interrupted());
        assert!(sub1_clone.is_interrupted());
    }
}
