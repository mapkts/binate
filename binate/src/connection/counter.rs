use std::sync::atomic::{AtomicI32, Ordering};

const COUNTER_MASK: u32 = 0x7FFFFFFF;

/// Atomic counter for counting request permits.
#[derive(Debug)]
pub struct RequestCounter(AtomicI32);

impl RequestCounter {
    /// Create a new `RequestCounter`.
    pub fn new(n: u32) -> Self {
        RequestCounter(AtomicI32::new((n & COUNTER_MASK) as i32))
    }

    /// Decrements this counter by 1, and returns the previous count.
    pub fn dec(&self) -> i32 {
        self.0.fetch_sub(1, Ordering::SeqCst)
    }

    /// Returns true if counter has reaching zero.
    pub fn is_zero(&self) -> bool {
        self.0.load(Ordering::SeqCst) == 0
    }

    /// Adds permits to this counter.
    pub fn add(&self, n: u32) {
        self.0.fetch_add(n as i32, Ordering::SeqCst);
    }

    /// Returns the count this counter stores.
    pub fn load(&self) -> i32 {
        self.0.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn assert_send_sync() {
        assert_send::<RequestCounter>();
        assert_sync::<RequestCounter>();
    }

    #[test]
    fn new() {
        let counter = RequestCounter::new(42);
        assert_eq!(counter.load(), 42);
    }

    #[test]
    fn dec() {
        let counter = RequestCounter::new(42);
        assert_eq!(counter.dec(), 42);
        assert_eq!(counter.load(), 41);
    }

    #[test]
    fn add() {
        let counter = RequestCounter::new(42);
        counter.add(8);
        assert_eq!(counter.load(), 50);
    }

    #[test]
    fn is_zero() {
        let counter = RequestCounter::new(1);
        counter.dec();
        assert!(counter.is_zero());
    }
}
