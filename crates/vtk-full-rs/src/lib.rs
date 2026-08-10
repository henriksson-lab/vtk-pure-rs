//! VTK-shaped Rust translation target.
//!
//! This crate starts intentionally small. Full VTK coverage is tracked by
//! `docs/audit/vtk_coverage.csv`; Rust stubs are created only when they help
//! active implementation work.

pub mod common;
pub mod filters;
pub mod imaging;
pub mod io;
pub mod rendering;
