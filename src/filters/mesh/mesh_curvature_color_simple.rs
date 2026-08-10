//! Color mesh vertices by curvature magnitude.
//!
//! Single implementation in [`crate::filters::mesh::vertex_coloring`]. It writes
//! the raw curvature as "Curvature" and the colours as a 0..1 "RGB" array (this
//! module used to emit a single 0..255 "CurvatureColor" array instead).

pub use crate::filters::mesh::vertex_coloring::color_by_curvature;
