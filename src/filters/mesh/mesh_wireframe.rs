//! Extract wireframe (all edges as lines) from a mesh.
//!
//! The single implementation lives in [`crate::filters::mesh::wireframe_extract`];
//! this module only re-exports it so the historical path keeps working.

pub use crate::filters::mesh::wireframe_extract::wireframe;
