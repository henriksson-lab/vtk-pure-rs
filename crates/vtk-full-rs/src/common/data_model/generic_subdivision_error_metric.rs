use std::{cell::RefCell, rc::Rc};

use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::GenericAttributeApi;

/// Rust boundary for `vtkGenericAdaptorCell*` methods used by translated
/// subdivision error metrics.
pub trait GenericAdaptorCellApi {
    /// VTK: `vtkGenericAdaptorCell::IsGeometryLinear`.
    fn is_geometry_linear(&self) -> bool;

    /// VTK: `vtkGenericAdaptorCell::IsAttributeLinear`.
    fn is_attribute_linear(&self, attribute: GenericAttributeHandle) -> bool;

    /// VTK: `vtkGenericAdaptorCell::GetId`.
    fn get_id(&self) -> VtkIdType;

    /// VTK: `vtkGenericAdaptorCell::GetType`.
    fn get_type(&self) -> i32;

    /// VTK: `vtkGenericAdaptorCell::GetDimension`.
    fn get_dimension(&self) -> i32;

    /// VTK: `vtkGenericAdaptorCell::GetNumberOfBoundaries`.
    fn get_number_of_boundaries(&self, dim: i32) -> i32;

    /// VTK: `vtkGenericAdaptorCell::GetPointIds`.
    fn get_point_ids(&self, out: &mut [VtkIdType]);

    /// VTK: `vtkGenericAdaptorCell::GetParametricCoords`.
    fn get_parametric_coords(&self) -> &[f64];

    /// VTK: `vtkGenericAdaptorCell::EvaluateLocation`.
    fn evaluate_location(&self, sub_id: i32, pcoords: [f64; 3]) -> [f64; 3];

    /// VTK: `vtkGenericAdaptorCell::InterpolateTuple`.
    fn interpolate_tuple_collection(
        &self,
        attributes: GenericAttributeCollectionHandle,
        pcoords: [f64; 3],
        out: &mut [f64],
    );

    /// VTK: `vtkGenericAdaptorCell::CountEdgeNeighbors`.
    fn count_edge_neighbors(&self, out: &mut [i32]);
}

/// Rust equivalent of the `vtkGenericAdaptorCell*` stored by
/// `vtkGenericSubdivisionErrorMetric`.
pub type GenericAdaptorCellHandle = Rc<RefCell<dyn GenericAdaptorCellApi>>;

/// VTK: `vtkGenericAttribute*`.
pub type GenericAttributeHandle = Rc<RefCell<dyn GenericAttributeApi>>;

/// Rust boundary for `vtkGenericAttributeCollection*` methods used by
/// translated subdivision error metrics.
pub trait GenericAttributeCollectionApi {
    /// VTK: `vtkGenericAttributeCollection::GetActiveAttribute`.
    fn get_active_attribute(&self) -> i32;

    /// VTK: `vtkGenericAttributeCollection::GetActiveComponent`.
    fn get_active_component(&self) -> i32;

    /// VTK: `vtkGenericAttributeCollection::GetAttribute`.
    fn get_attribute(&self, index: i32) -> Option<GenericAttributeHandle>;

    /// VTK: `vtkGenericAttributeCollection::GetAttributeIndex`.
    fn get_attribute_index(&self, index: i32) -> i32;

    /// VTK: `vtkGenericAttributeCollection::GetNumberOfComponents`.
    fn get_number_of_components(&self) -> i32;
}

/// VTK: `vtkGenericAttributeCollection*`.
pub type GenericAttributeCollectionHandle = Rc<RefCell<dyn GenericAttributeCollectionApi>>;

/// Rust boundary for `vtkGenericDataSet*` methods used by translated
/// subdivision error metrics.
pub trait GenericDataSetApi {
    /// VTK: `vtkGenericDataSet::GetNumberOfPoints`.
    fn get_number_of_points(&self) -> VtkIdType;

    /// VTK: `vtkGenericDataSet::GetBounds`.
    fn get_bounds(&self) -> [f64; 6];

    /// VTK: `vtkGenericDataSet::GetLength`.
    fn get_length(&self) -> f64;

    /// VTK: `vtkGenericDataSet::GetAttributes`.
    fn get_attributes(&self) -> Option<GenericAttributeCollectionHandle>;
}

/// VTK: `vtkGenericDataSet*`.
pub type GenericDataSetHandle = Rc<RefCell<dyn GenericDataSetApi>>;

/// VTK: `vtkGenericSubdivisionErrorMetric`.
///
/// This stores the abstract VTK base-class state. The adaptor-cell and generic
/// dataset classes are forward declarations in this VTK header and remain
/// opaque until translated.
#[derive(Clone)]
pub struct GenericSubdivisionErrorMetric {
    object: Object,
    generic_cell: Option<GenericAdaptorCellHandle>,
    data_set: Option<GenericDataSetHandle>,
}

impl GenericSubdivisionErrorMetric {
    /// VTK: `vtkGenericSubdivisionErrorMetric::vtkGenericSubdivisionErrorMetric`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            generic_cell: None,
            data_set: None,
        }
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "GenericCell: {:?}\nDataSet: {:?}\n",
            self.generic_cell.as_ref().map(Rc::as_ptr),
            self.data_set.as_ref().map(Rc::as_ptr)
        )
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetGenericCell`.
    pub fn set_generic_cell(&mut self, cell: Option<GenericAdaptorCellHandle>) {
        if option_cell_ptr_eq(&self.generic_cell, &cell) {
            return;
        }
        self.generic_cell = cell;
        self.modified();
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetGenericCell`.
    pub fn get_generic_cell(&self) -> Option<GenericAdaptorCellHandle> {
        self.generic_cell.clone()
    }

    pub(crate) fn is_geometry_linear(&self) -> bool {
        self.generic_cell
            .as_ref()
            .is_some_and(|cell| cell.borrow().is_geometry_linear())
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: Option<GenericDataSetHandle>) {
        if option_data_set_ptr_eq(&self.data_set, &data_set) {
            return;
        }
        self.data_set = data_set;
        self.modified();
    }

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetDataSet`.
    pub fn get_data_set(&self) -> Option<GenericDataSetHandle> {
        self.data_set.clone()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl std::fmt::Debug for GenericSubdivisionErrorMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericSubdivisionErrorMetric")
            .field("object", &self.object)
            .field("generic_cell", &self.generic_cell.as_ref().map(Rc::as_ptr))
            .field("data_set", &self.data_set.as_ref().map(Rc::as_ptr))
            .finish()
    }
}

impl Default for GenericSubdivisionErrorMetric {
    fn default() -> Self {
        Self::with_class_name("vtkGenericSubdivisionErrorMetric")
    }
}

/// VTK pure virtual API for `vtkGenericSubdivisionErrorMetric`.
pub trait GenericSubdivisionErrorMetricApi {
    /// VTK: `vtkGenericSubdivisionErrorMetric::SetGenericCell`.
    fn set_generic_cell(&mut self, cell: Option<GenericAdaptorCellHandle>);

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetGenericCell`.
    fn get_generic_cell(&self) -> Option<GenericAdaptorCellHandle>;

    /// VTK: `vtkGenericSubdivisionErrorMetric::SetDataSet`.
    fn set_data_set(&mut self, data_set: Option<GenericDataSetHandle>);

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetDataSet`.
    fn get_data_set(&self) -> Option<GenericDataSetHandle>;

    /// VTK: `vtkGenericSubdivisionErrorMetric::RequiresEdgeSubdivision`.
    fn requires_edge_subdivision(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) -> i32;

    /// VTK: `vtkGenericSubdivisionErrorMetric::GetError`.
    fn get_error(
        &mut self,
        left_point: &mut [f64],
        mid_point: &mut [f64],
        right_point: &mut [f64],
        alpha: f64,
    ) -> f64;
}

fn option_cell_ptr_eq(
    left: &Option<GenericAdaptorCellHandle>,
    right: &Option<GenericAdaptorCellHandle>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn option_data_set_ptr_eq(
    left: &Option<GenericDataSetHandle>,
    right: &Option<GenericDataSetHandle>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}
