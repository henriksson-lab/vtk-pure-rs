//! Generic dataset enum that wraps any concrete dataset type.
//!
//! `AnyDataSet` provides a unified type for working with different dataset
//! types through the `DataSet` trait interface.

use crate::types::BoundingBox;

use crate::data::traits::{DataObject, DataSet};
use crate::data::{
    DataSetAttributes, ExplicitStructuredGrid, FieldData, ImageData, PolyData, RectilinearGrid,
    StructuredGrid, UnstructuredGrid,
};

/// Enum wrapping all concrete dataset types.
///
/// Provides delegated access to `DataSet` trait methods without dynamic dispatch.
#[derive(Debug, Clone)]
pub enum AnyDataSet {
    /// Polygonal mesh.
    Poly(PolyData),
    /// Regular image grid.
    Image(ImageData),
    /// Unstructured mixed-cell mesh.
    Unstructured(UnstructuredGrid),
    /// Axis-aligned grid with non-uniform spacing.
    Rectilinear(RectilinearGrid),
    /// Curvilinear structured grid.
    Structured(StructuredGrid),
    /// Explicit structured grid.
    ExplicitStructured(ExplicitStructuredGrid),
}

impl AnyDataSet {
    /// Number of points in the dataset.
    pub fn num_points(&self) -> usize {
        match self {
            Self::Poly(d) => d.num_points(),
            Self::Image(d) => d.num_points(),
            Self::Unstructured(d) => d.num_points(),
            Self::Rectilinear(d) => d.num_points(),
            Self::Structured(d) => d.num_points(),
            Self::ExplicitStructured(d) => d.num_points(),
        }
    }

    /// Number of cells in the dataset.
    pub fn num_cells(&self) -> usize {
        match self {
            Self::Poly(d) => d.num_cells(),
            Self::Image(d) => d.num_cells(),
            Self::Unstructured(d) => d.num_cells(),
            Self::Rectilinear(d) => d.num_cells(),
            Self::Structured(d) => d.num_cells(),
            Self::ExplicitStructured(d) => d.num_cells(),
        }
    }

    /// Get coordinates of a point by index.
    pub fn point(&self, idx: usize) -> [f64; 3] {
        match self {
            Self::Poly(d) => d.point(idx),
            Self::Image(d) => d.point(idx),
            Self::Unstructured(d) => d.point(idx),
            Self::Rectilinear(d) => d.point(idx),
            Self::Structured(d) => d.point(idx),
            Self::ExplicitStructured(d) => d.points.get(idx),
        }
    }

    /// Bounding box of the dataset.
    pub fn bounds(&self) -> BoundingBox {
        match self {
            Self::Poly(d) => d.bounds(),
            Self::Image(d) => d.bounds(),
            Self::Unstructured(d) => d.bounds(),
            Self::Rectilinear(d) => d.bounds(),
            Self::Structured(d) => d.bounds(),
            Self::ExplicitStructured(d) => d.bounds(),
        }
    }

    /// Point data attributes.
    pub fn point_data(&self) -> &DataSetAttributes {
        match self {
            Self::Poly(d) => d.point_data(),
            Self::Image(d) => d.point_data(),
            Self::Unstructured(d) => d.point_data(),
            Self::Rectilinear(d) => d.point_data(),
            Self::Structured(d) => d.point_data(),
            Self::ExplicitStructured(d) => d.point_data(),
        }
    }

    /// Cell data attributes.
    pub fn cell_data(&self) -> &DataSetAttributes {
        match self {
            Self::Poly(d) => d.cell_data(),
            Self::Image(d) => d.cell_data(),
            Self::Unstructured(d) => d.cell_data(),
            Self::Rectilinear(d) => d.cell_data(),
            Self::Structured(d) => d.cell_data(),
            Self::ExplicitStructured(d) => d.cell_data(),
        }
    }
}

impl DataObject for AnyDataSet {
    fn field_data(&self) -> &FieldData {
        match self {
            Self::Poly(d) => d.field_data(),
            Self::Image(d) => d.field_data(),
            Self::Unstructured(d) => d.field_data(),
            Self::Rectilinear(d) => d.field_data(),
            Self::Structured(d) => d.field_data(),
            Self::ExplicitStructured(d) => d.field_data(),
        }
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        match self {
            Self::Poly(d) => d.field_data_mut(),
            Self::Image(d) => d.field_data_mut(),
            Self::Unstructured(d) => d.field_data_mut(),
            Self::Rectilinear(d) => d.field_data_mut(),
            Self::Structured(d) => d.field_data_mut(),
            Self::ExplicitStructured(d) => d.field_data_mut(),
        }
    }
}

impl DataSet for AnyDataSet {
    fn num_points(&self) -> usize {
        self.num_points()
    }

    fn num_cells(&self) -> usize {
        self.num_cells()
    }

    fn point(&self, idx: usize) -> [f64; 3] {
        self.point(idx)
    }

    fn bounds(&self) -> BoundingBox {
        self.bounds()
    }

    fn point_data(&self) -> &DataSetAttributes {
        self.point_data()
    }

    fn point_data_mut(&mut self) -> &mut DataSetAttributes {
        match self {
            Self::Poly(d) => d.point_data_mut(),
            Self::Image(d) => d.point_data_mut(),
            Self::Unstructured(d) => d.point_data_mut(),
            Self::Rectilinear(d) => d.point_data_mut(),
            Self::Structured(d) => d.point_data_mut(),
            Self::ExplicitStructured(d) => d.point_data_mut(),
        }
    }

    fn cell_data(&self) -> &DataSetAttributes {
        self.cell_data()
    }

    fn cell_data_mut(&mut self) -> &mut DataSetAttributes {
        match self {
            Self::Poly(d) => d.cell_data_mut(),
            Self::Image(d) => d.cell_data_mut(),
            Self::Unstructured(d) => d.cell_data_mut(),
            Self::Rectilinear(d) => d.cell_data_mut(),
            Self::Structured(d) => d.cell_data_mut(),
            Self::ExplicitStructured(d) => d.cell_data_mut(),
        }
    }
}

impl From<PolyData> for AnyDataSet {
    fn from(d: PolyData) -> Self {
        Self::Poly(d)
    }
}

impl From<ImageData> for AnyDataSet {
    fn from(d: ImageData) -> Self {
        Self::Image(d)
    }
}

impl From<UnstructuredGrid> for AnyDataSet {
    fn from(d: UnstructuredGrid) -> Self {
        Self::Unstructured(d)
    }
}

impl From<RectilinearGrid> for AnyDataSet {
    fn from(d: RectilinearGrid) -> Self {
        Self::Rectilinear(d)
    }
}

impl From<StructuredGrid> for AnyDataSet {
    fn from(d: StructuredGrid) -> Self {
        Self::Structured(d)
    }
}

impl From<ExplicitStructuredGrid> for AnyDataSet {
    fn from(d: ExplicitStructuredGrid) -> Self {
        Self::ExplicitStructured(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_dataset_from_poly_data() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let any = AnyDataSet::from(pd);
        assert_eq!(any.num_points(), 3);
        assert_eq!(any.num_cells(), 1);
        let p = any.point(1);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn any_dataset_from_image_data() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let any = AnyDataSet::from(img);
        assert_eq!(any.num_points(), 27);
        let b = any.bounds();
        assert!(b.diagonal_length() > 0.0);
    }

    #[test]
    fn any_dataset_from_explicit_structured_grid() {
        let grid = ExplicitStructuredGrid::new([2, 2, 2]);
        let any = AnyDataSet::from(grid);
        assert_eq!(any.num_points(), 8);
        assert_eq!(any.num_cells(), 1);
    }
}
