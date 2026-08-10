//! Detect sharp edges and feature lines on meshes.
//!
//! The single implementation lives in [`crate::filters::mesh::sharp_edges`]; this
//! module only re-exports it so the historical path keeps working.

/// Extract edges with dihedral angle above threshold (in degrees).
pub use crate::filters::mesh::sharp_edges::extract_sharp_edges;
