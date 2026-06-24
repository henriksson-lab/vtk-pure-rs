//! Simple timer for measuring filter execution time.

use std::time::Instant;

/// A simple timer for measuring elapsed time.
///
/// ```
/// use crate::types::timer::Timer;
///
/// let timer = Timer::start();
/// // ... do work ...
/// let elapsed = timer.elapsed_ms();
/// ```
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Start a new timer.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            label: String::new(),
        }
    }

    /// Start a named timer.
    pub fn named(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    /// Elapsed time in seconds.
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Print elapsed time to stdout and return the elapsed ms.
    pub fn print(&self) -> f64 {
        let ms = self.elapsed_ms();
        if self.label.is_empty() {
            println!("Elapsed: {ms:.2} ms");
        } else {
            println!("{}: {ms:.2} ms", self.label);
        }
        ms
    }

    /// Reset the timer.
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Time a closure and return (result, elapsed_ms).
    pub fn time<T>(f: impl FnOnce() -> T) -> (T, f64) {
        let t = Self::start();
        let result = f();
        (result, t.elapsed_ms())
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.elapsed_ms();
        if ms < 1.0 {
            write!(f, "{:.1} µs", ms * 1000.0)
        } else if ms < 1000.0 {
            write!(f, "{:.2} ms", ms)
        } else {
            write!(f, "{:.2} s", ms / 1000.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_timer() {
        let t = Timer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(t.elapsed_ms() >= 5.0);
    }

    #[test]
    fn named_timer() {
        let t = Timer::named("test");
        assert!(t.elapsed_ms() >= 0.0);
        let s = format!("{t}");
        assert!(!s.is_empty());
    }

    #[test]
    fn time_closure() {
        let (result, ms) = Timer::time(|| 42);
        assert_eq!(result, 42);
        assert!(ms >= 0.0);
    }
}
