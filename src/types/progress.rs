//! Progress reporting for long-running operations.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// A progress reporter that filters can update during execution.
///
/// Thread-safe: can be shared across rayon parallel operations.
///
/// ```
/// use crate::types::progress::Progress;
///
/// let progress = Progress::new();
/// progress.set_total(100);
/// progress.set_current(50);
/// assert_eq!(progress.fraction(), 0.5);
/// ```
#[derive(Clone)]
pub struct Progress {
    inner: Arc<ProgressInner>,
}

struct ProgressInner {
    current: AtomicU64,
    total: AtomicU64,
    cancelled: AtomicBool,
    message: std::sync::Mutex<String>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ProgressInner {
                current: AtomicU64::new(0),
                total: AtomicU64::new(100),
                cancelled: AtomicBool::new(false),
                message: std::sync::Mutex::new(String::new()),
            }),
        }
    }

    /// Set the total number of steps.
    pub fn set_total(&self, total: u64) {
        self.inner.total.store(total, Ordering::Relaxed);
    }

    /// Set the current progress.
    pub fn set_current(&self, current: u64) {
        self.inner.current.store(current, Ordering::Relaxed);
    }

    /// Increment progress by 1.
    pub fn increment(&self) {
        self.inner.current.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current progress as a fraction [0, 1].
    pub fn fraction(&self) -> f64 {
        let total = self.inner.total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let current = self.inner.current.load(Ordering::Relaxed);
        current as f64 / total as f64
    }

    /// Get current progress as a percentage [0, 100].
    pub fn percentage(&self) -> f64 {
        self.fraction() * 100.0
    }

    /// Set a status message.
    pub fn set_message(&self, msg: impl Into<String>) {
        *self.inner.message.lock().unwrap() = msg.into();
    }

    /// Get the current status message.
    pub fn message(&self) -> String {
        self.inner.message.lock().unwrap().clone()
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::Relaxed);
    }

    /// Check if cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Relaxed)
    }

    /// Whether the operation is complete.
    pub fn is_done(&self) -> bool {
        let current = self.inner.current.load(Ordering::Relaxed);
        let total = self.inner.total.load(Ordering::Relaxed);
        current >= total
    }

    /// Reset progress to zero.
    pub fn reset(&self) {
        self.inner.current.store(0, Ordering::Relaxed);
        self.inner.cancelled.store(false, Ordering::Relaxed);
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.message();
        if msg.is_empty() {
            write!(f, "{:.1}%", self.percentage())
        } else {
            write!(f, "{:.1}% - {}", self.percentage(), msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_progress() {
        let p = Progress::new();
        p.set_total(10);
        assert_eq!(p.fraction(), 0.0);
        p.set_current(5);
        assert!((p.fraction() - 0.5).abs() < 1e-10);
        assert!((p.percentage() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn increment() {
        let p = Progress::new();
        p.set_total(3);
        p.increment();
        p.increment();
        assert!((p.fraction() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn cancellation() {
        let p = Progress::new();
        assert!(!p.is_cancelled());
        p.cancel();
        assert!(p.is_cancelled());
    }

    #[test]
    fn message() {
        let p = Progress::new();
        p.set_message("Processing normals");
        assert_eq!(p.message(), "Processing normals");
    }

    #[test]
    fn display() {
        let p = Progress::new();
        p.set_total(4);
        p.set_current(1);
        let s = format!("{p}");
        assert!(s.contains("25.0%"));
    }

    #[test]
    fn thread_safe() {
        let p = Progress::new();
        p.set_total(1000);
        let p2 = p.clone();
        std::thread::spawn(move || {
            for _ in 0..500 {
                p2.increment();
            }
        })
        .join()
        .unwrap();
        assert!(p.fraction() > 0.0);
    }
}
