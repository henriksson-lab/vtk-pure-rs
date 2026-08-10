//! Merge vertices that are closer than a threshold distance.
//!
//! The implementation lives in [`crate::filters::mesh::vertex_merge_by_distance`],
//! which follows `vtkCleanPolyData` (k-d tree accelerated point merging plus
//! VTK's degenerate-cell handling). This module only re-exports it.

pub use crate::filters::mesh::vertex_merge_by_distance::merge_close_vertices;
