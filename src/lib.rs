//! vtk-rs — a pure Rust visualization toolkit.

// Always available
pub mod data;
pub mod filters;
pub mod types;

// I/O (feature-gated)
#[cfg(feature = "io-common")]
pub mod io;

// Rendering (feature-gated)
#[cfg(feature = "render")]
pub mod render;
#[cfg(feature = "render-wgpu")]
pub mod render_wgpu;

// Parallel (feature-gated)
#[cfg(feature = "parallel")]
pub mod parallel;

// Convenience re-exports (always available)
pub use data::{
    AnyDataArray, CellArray, DataArray, DataObject, DataSet, DataSetAttributes, FieldData,
    ImageData, KdTree, MultiBlockDataSet, Points, PolyData, RectilinearGrid, Selection,
    StructuredGrid, Table, UnstructuredGrid,
};
pub use types::{BoundingBox, CellType, Scalar, ScalarType, VtkError};

pub mod prelude {
    pub use crate::data::prelude::*;
    pub use crate::{BoundingBox, CellType, VtkError};
}
