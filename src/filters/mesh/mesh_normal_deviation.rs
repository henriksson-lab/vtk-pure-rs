//! Compute deviation angle between vertex normal and adjacent face normals.
//!
//! Re-exported from [`crate::filters::mesh::vertex_normal_deviation`], which
//! holds the single implementation (it also takes triangle strips into
//! account, which this module's former copy did not).

pub use crate::filters::mesh::vertex_normal_deviation::normal_deviation;
