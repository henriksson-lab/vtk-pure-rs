//! Gradient magnitude of a scalar field on a mesh.
//!
//! Re-exported from [`crate::filters::mesh::mesh_array_gradient`], which holds
//! the single implementation: the exact per-triangle linear-element gradient
//! (`vtkTriangle::Derivatives`, as used by `vtkGradientFilter`) averaged onto
//! the vertices, rather than the inverse-distance finite difference this
//! module used to carry. The output arrays are "Gradient" (3 components) and
//! the active scalar "GradMagnitude".

pub use crate::filters::mesh::mesh_array_gradient::scalar_gradient_magnitude;
