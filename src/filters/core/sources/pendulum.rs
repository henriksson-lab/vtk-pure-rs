//! Simple pendulum (bob on string from pivot).
//!
//! The single implementation lives in
//! [`crate::filters::core::sources::pendulum_model`]; this module re-exports it
//! so the historical `sources::pendulum::pendulum` path keeps working.

pub use crate::filters::core::sources::pendulum_model::pendulum;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pendulum() {
        let m = pendulum(5.0, 0.3, 30.0, 8);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() > 10);
        assert_eq!(m.lines.num_cells(), 1);
    }
}
