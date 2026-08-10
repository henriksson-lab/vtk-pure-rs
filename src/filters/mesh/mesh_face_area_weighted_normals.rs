//! Compute face-area-weighted vertex normals.
//!
//! The single implementation lives in [`crate::filters::mesh::area_weighted_normals`]
//! (the faithful `vtkTriangleMeshPointNormals` translation); this module only
//! re-exports it so the historical path keeps working.

pub use crate::filters::mesh::area_weighted_normals::area_weighted_normals;
