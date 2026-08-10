use super::{
    BoundingBox, DataObject, DataObjectType, DataSetAttributes, DataSetCellIterator, CELL, POINT,
};
use crate::common::core::{IdList, VtkIdType};

use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct DataSetStorage {
    data_object: DataObject,
    point_data: DataSetAttributes,
    cell_data: DataSetAttributes,
    bounds: BoundingBox,
    scalar_range: [f64; 2],
    modified_time: u64,
}

/// VTK-shaped base for `vtkDataSet`.
///
/// Concrete datasets still own their geometry/topology. This base contains the
/// common attribute payload and cached metadata helpers used by those datasets.
#[derive(Debug)]
pub struct DataSet {
    storage: Arc<DataSetStorage>,
}

impl Clone for DataSet {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl PartialEq for DataSet {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage) || self.storage == other.storage
    }
}

impl DataSet {
    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            storage: Arc::new(DataSetStorage {
                data_object: DataObject::with_type(object_type),
                point_data: DataSetAttributes::new(),
                cell_data: DataSetAttributes::new(),
                bounds: BoundingBox::empty(),
                scalar_range: [0.0, 1.0],
                modified_time: 0,
            }),
        }
    }

    fn storage_mut(&mut self) -> &mut DataSetStorage {
        Arc::make_mut(&mut self.storage)
    }

    pub(crate) fn data_object(&self) -> &DataObject {
        &self.storage.data_object
    }

    #[cfg(test)]
    pub(crate) fn data_object_mut(&mut self) -> &mut DataObject {
        self.modified();
        &mut self.storage_mut().data_object
    }

    /// VTK: `vtkDataSet::GetPointData`.
    pub fn get_point_data(&self) -> &DataSetAttributes {
        &self.storage.point_data
    }

    pub(crate) fn get_point_data_mut(&mut self) -> &mut DataSetAttributes {
        self.modified();
        &mut self.storage_mut().point_data
    }

    /// VTK: `vtkDataSet::GetCellData`.
    pub fn get_cell_data(&self) -> &DataSetAttributes {
        &self.storage.cell_data
    }

    pub(crate) fn get_cell_data_mut(&mut self) -> &mut DataSetAttributes {
        self.modified();
        &mut self.storage_mut().cell_data
    }

    /// VTK: `vtkDataSet::CopyAttributes`.
    pub fn copy_attributes(&mut self, source: &Self) {
        let storage = self.storage_mut();
        storage.point_data.pass_data(&source.storage.point_data);
        storage.cell_data.pass_data(&source.storage.cell_data);
        storage
            .data_object
            .get_field_data_mut()
            .pass_data(source.storage.data_object.get_field_data());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataSet::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.data_object.initialize();
        storage.point_data.initialize();
        storage.cell_data.initialize();
        storage.bounds = BoundingBox::empty();
        storage.scalar_range = [0.0, 1.0];
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataSet::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.data_object.get_field_data_mut().squeeze();
        storage.point_data.squeeze();
        storage.cell_data.squeeze();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataSet::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.storage.bounds.get_bounds()
    }

    pub(crate) fn set_bounds(&mut self, bounds: BoundingBox) {
        let storage = self.storage_mut();
        storage.bounds = bounds;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataSet::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        let bounds = self.get_bounds();
        [
            (bounds[0] + bounds[1]) * 0.5,
            (bounds[2] + bounds[3]) * 0.5,
            (bounds[4] + bounds[5]) * 0.5,
        ]
    }

    /// VTK: `vtkDataSet::GetScalarRange`.
    pub fn get_scalar_range(&self) -> [f64; 2] {
        match (
            self.get_point_data()
                .get_field_data_scalars()
                .and_then(compute_scalar_range),
            self.get_cell_data()
                .get_field_data_scalars()
                .and_then(compute_scalar_range),
        ) {
            (Some(point_range), Some(cell_range)) => [
                point_range[0].min(cell_range[0]),
                point_range[1].max(cell_range[1]),
            ],
            (Some(point_range), None) => point_range,
            (None, Some(cell_range)) => cell_range,
            (None, None) => self.storage.scalar_range,
        }
    }

    #[cfg(test)]
    pub(crate) fn set_scalar_range(&mut self, scalar_range: [f64; 2]) {
        let storage = self.storage_mut();
        storage.scalar_range = scalar_range;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.data_object.get_actual_memory_size()
            + self.storage.point_data.get_actual_memory_size()
            + self.storage.cell_data.get_actual_memory_size()
    }

    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.data_object.get_m_time())
            .max(self.storage.point_data.get_m_time())
            .max(self.storage.cell_data.get_m_time())
    }

    /// VTK: `vtkDataSet::GetAttributesAsFieldData`.
    pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&super::FieldData> {
        match attribute_type {
            POINT => Some(self.get_point_data().field_data()),
            CELL => Some(self.get_cell_data().field_data()),
            _ => self
                .storage
                .data_object
                .get_attributes_as_field_data(attribute_type),
        }
    }

    /// VTK: `vtkDataSet::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        self.storage
            .data_object
            .get_number_of_elements(attribute_type)
    }

    pub fn modified(&mut self) {
        let next = self.storage.modified_time.saturating_add(1);
        self.storage_mut().modified_time = next;
    }

    pub fn shallow_copy(&mut self, source: &Self) {
        let mut data_object = self.storage.data_object.shallow_clone();
        data_object.shallow_copy(&source.storage.data_object);
        self.storage = Arc::new(DataSetStorage {
            data_object,
            point_data: source.storage.point_data.shallow_clone(),
            cell_data: source.storage.cell_data.shallow_clone(),
            bounds: source.storage.bounds,
            scalar_range: source.storage.scalar_range,
            modified_time: self.storage.modified_time.saturating_add(1),
        });
    }

    pub fn deep_copy(&mut self, source: &Self) {
        let mut data_object = self.storage.data_object.deep_clone();
        data_object.deep_copy(&source.storage.data_object);
        self.storage = Arc::new(DataSetStorage {
            data_object,
            point_data: source.storage.point_data.deep_clone(),
            cell_data: source.storage.cell_data.deep_clone(),
            bounds: source.storage.bounds,
            scalar_range: source.storage.scalar_range,
            modified_time: self.storage.modified_time.saturating_add(1),
        });
    }

    /// VTK: `vtkDataSet::NewCellIterator`.
    pub fn new_cell_iterator<'a>(data_set: &'a mut dyn DataSetApi) -> DataSetCellIterator<'a> {
        let mut iter = DataSetCellIterator::new();
        iter.set_data_set(Some(data_set));
        iter
    }
}

fn compute_scalar_range(array: &super::FieldDataArray) -> Option<[f64; 2]> {
    if !array.get_data().is_numeric() || array.get_number_of_tuples() == 0 {
        return None;
    }

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for tuple_idx in 0..array.get_number_of_tuples() {
        let tuple = array
            .get_data()
            .numeric_tuple_as_f64_checked(tuple_idx)
            .ok()?;
        let value = tuple.first().copied()?;
        min = min.min(value);
        max = max.max(value);
    }
    Some([min, max])
}

/// VTK virtual dataset methods needed by `vtkDataSetCellIterator`.
pub trait DataSetApi {
    fn data_set(&self) -> &DataSet;
    fn data_set_mut(&mut self) -> &mut DataSet;

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str;

    /// VTK: `vtkDataSet::GetNumberOfCells`.
    fn get_number_of_cells(&self) -> VtkIdType;

    /// VTK: `vtkDataSet::GetNumberOfPoints`.
    fn get_number_of_points(&self) -> VtkIdType;

    /// VTK: `vtkDataSet::GetCellType`.
    fn get_cell_type(&self, cell_id: VtkIdType) -> i32;

    /// VTK: `vtkDataSet::GetCellPoints(vtkIdType, vtkIdList*)`.
    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList);

    /// VTK: `vtkDataSet::GetPoint(vtkIdType, double[3])`.
    fn get_point(&self, point_id: VtkIdType) -> [f64; 3];

    /// VTK local helper equivalent: coordinate array data types for
    /// `vtkDataSetCellIterator` SetArrayType.
    fn coordinate_data_types(&self) -> [Option<i32>; 3] {
        [None, None, None]
    }
}
