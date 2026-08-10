use super::{
    AbstractCellLocatorHandle, BoundingBox, CellType, DataObjectType, DataSet, DataSetApi,
    PointSetCellIterator, CELL, POINT,
};
use crate::common::core::{IdList, Points, VtkIdType};

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
struct PointSetStorage {
    data_set: DataSet,
    points: Option<Points>,
    cell_locator: Option<AbstractCellLocatorHandle>,
    editable: bool,
    modified_time: u64,
}

impl fmt::Debug for PointSetStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointSetStorage")
            .field("data_set", &self.data_set)
            .field("points", &self.points)
            .field("cell_locator", &self.cell_locator.is_some())
            .field("editable", &self.editable)
            .field("modified_time", &self.modified_time)
            .finish()
    }
}

impl PartialEq for PointSetStorage {
    fn eq(&self, other: &Self) -> bool {
        self.data_set == other.data_set
            && self.points == other.points
            && match (&self.cell_locator, &other.cell_locator) {
                (Some(lhs), Some(rhs)) => Rc::ptr_eq(lhs, rhs),
                (None, None) => true,
                _ => false,
            }
            && self.editable == other.editable
            && self.modified_time == other.modified_time
    }
}

/// VTK-shaped base for `vtkPointSet`.
///
/// This owns the nullable `vtkPoints*` equivalent and the common dataset
/// attributes. Concrete subclasses still own their cell topology.
#[derive(Debug)]
pub struct PointSet {
    storage: Arc<PointSetStorage>,
}

impl Clone for PointSet {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl PartialEq for PointSet {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage) || self.storage == other.storage
    }
}

impl PointSet {
    pub fn new() -> Self {
        Self::with_type(DataObjectType::PointSet)
    }

    /// VTK: `vtkPointSet::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            storage: Arc::new(PointSetStorage {
                data_set: DataSet::with_type(object_type),
                points: None,
                cell_locator: None,
                editable: false,
                modified_time: 0,
            }),
        }
    }

    fn storage_mut(&mut self) -> &mut PointSetStorage {
        Arc::make_mut(&mut self.storage)
    }

    pub(crate) fn data_set(&self) -> &DataSet {
        &self.storage.data_set
    }

    pub(crate) fn data_set_mut(&mut self) -> &mut DataSet {
        &mut self.storage_mut().data_set
    }

    /// VTK: `vtkPointSet::GetEditable`.
    pub fn get_editable(&self) -> bool {
        self.storage.editable
    }

    /// VTK: `vtkPointSet::SetEditable`.
    pub fn set_editable(&mut self, editable: bool) {
        if self.storage.editable != editable {
            let storage = self.storage_mut();
            storage.editable = editable;
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
    }

    /// VTK: `vtkPointSet::EditableOn`.
    pub fn editable_on(&mut self) {
        self.set_editable(true);
    }

    /// VTK: `vtkPointSet::EditableOff`.
    pub fn editable_off(&mut self) {
        self.set_editable(false);
    }

    /// VTK: `vtkPointSet::GetPoints`.
    pub fn get_points(&self) -> Option<&Points> {
        self.storage.points.as_ref()
    }

    /// VTK: `vtkPointSet::SetPoints`.
    pub fn set_points(&mut self, points: Option<Points>) {
        let bounds = points
            .as_ref()
            .map(|points| BoundingBox::from_bounds(points.get_bounds()))
            .unwrap_or_else(BoundingBox::empty);
        let storage = self.storage_mut();
        storage.points = points;
        storage.data_set.set_bounds(bounds);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataSet::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.storage
            .points
            .as_ref()
            .map_or(0, Points::get_number_of_points)
    }

    /// Base `vtkPointSet` has no concrete cells.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        0
    }

    /// VTK: `vtkPointSet::GetCellPoints`.
    pub fn get_cell_points(&self, _cell_id: VtkIdType, id_list: &mut IdList) {
        id_list.reset();
    }

    /// VTK: `vtkPointSet::GetPointCells`.
    pub fn get_point_cells(&self, _point_id: VtkIdType, id_list: &mut IdList) {
        id_list.reset();
    }

    /// VTK: `vtkPointSet::GetCellType`.
    pub fn get_cell_type(&self, _cell_id: VtkIdType) -> i32 {
        CellType::Empty as i32
    }

    /// VTK: `vtkPointSet::GetCellSize`.
    pub fn get_cell_size(&self, _cell_id: VtkIdType) -> VtkIdType {
        1
    }

    /// VTK: `vtkPointSet::NewCellIterator`.
    pub fn new_cell_iterator(&mut self) -> PointSetCellIterator {
        let mut iter = PointSetCellIterator::new();
        iter.set_point_set(self as *mut Self);
        iter
    }

    /// VTK: `vtkPointSet::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        0
    }

    /// VTK: `vtkDataSet::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            POINT => self.get_number_of_points(),
            CELL => self.get_number_of_cells(),
            _ => self.storage.data_set.get_number_of_elements(attribute_type),
        }
    }

    /// VTK: `vtkDataSet::GetPoint`.
    pub fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        self.storage
            .points
            .as_ref()
            .expect("vtkPointSet points exist")
            .get_point(point_id)
    }

    /// VTK: `vtkPointSet::BuildCellLocator`.
    pub fn build_cell_locator(&mut self) {
        if self.storage.points.is_none() {
            return;
        }

        if let Some(locator) = self.storage.cell_locator.clone() {
            locator.borrow_mut().set_data_set(self as *mut PointSet);
            locator.borrow_mut().build_locator();
        }
    }

    /// VTK: `vtkPointSet::SetCellLocator`.
    pub fn set_cell_locator(&mut self, cell_locator: Option<AbstractCellLocatorHandle>) {
        let changed = match (&self.storage.cell_locator, &cell_locator) {
            (Some(current), Some(next)) => !Rc::ptr_eq(current, next),
            (None, None) => false,
            _ => true,
        };
        if changed {
            let storage = self.storage_mut();
            storage.cell_locator = cell_locator;
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
    }

    /// VTK: `vtkPointSet::GetCellLocator`.
    pub fn get_cell_locator(&self) -> Option<AbstractCellLocatorHandle> {
        self.storage.cell_locator.as_ref().map(Rc::clone)
    }

    /// VTK: `vtkPointSet::ComputeBounds`.
    pub fn compute_bounds(&mut self) {
        let storage = self.storage_mut();
        let bounds = if let Some(points) = storage.points.as_mut() {
            points.compute_bounds();
            BoundingBox::from_bounds(points.get_bounds())
        } else {
            BoundingBox::empty()
        };
        storage.data_set.set_bounds(bounds);
    }

    /// VTK: `vtkPointSet::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.storage
            .points
            .as_ref()
            .map(Points::get_bounds)
            .unwrap_or_else(|| BoundingBox::empty().get_bounds())
    }

    /// VTK: `vtkPointSet::CopyStructure`.
    pub fn copy_structure(&mut self, source: &Self) {
        if self.storage.points != source.storage.points {
            if let Some(locator) = self.storage.cell_locator.as_ref() {
                locator.borrow_mut().initialize();
            }
            self.set_points(source.storage.points.clone());
        }
    }

    /// VTK: `vtkPointSet::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.data_set.initialize();
        storage.points = None;
        if let Some(locator) = storage.cell_locator.as_ref() {
            locator.borrow_mut().initialize();
        }
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPointSet::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.data_set.squeeze();
        if let Some(points) = storage.points.as_mut() {
            points.squeeze();
        }
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPointSet::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.data_set.get_actual_memory_size()
            + self
                .storage
                .points
                .as_ref()
                .map_or(0, Points::get_actual_memory_size)
    }

    /// VTK: `vtkObject::GetMTime` with points included.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.data_set.get_m_time())
            .max(self.storage.points.as_ref().map_or(0, Points::get_m_time))
    }

    pub fn modified(&mut self) {
        let next = self.storage.modified_time.saturating_add(1);
        self.storage_mut().modified_time = next;
    }

    /// VTK: `vtkPointSet::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        let modified_time = self.storage.modified_time.saturating_add(1);
        self.storage = Arc::new(PointSetStorage {
            data_set: {
                let mut data_set =
                    DataSet::with_type(source.storage.data_set.data_object().data_object_type());
                data_set.shallow_copy(&source.storage.data_set);
                data_set
            },
            points: source.storage.points.clone(),
            cell_locator: None,
            editable: source.storage.editable,
            modified_time,
        });
    }

    /// VTK: `vtkPointSet::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        let modified_time = self.storage.modified_time.saturating_add(1);
        self.storage = Arc::new(PointSetStorage {
            data_set: {
                let mut data_set =
                    DataSet::with_type(source.storage.data_set.data_object().data_object_type());
                data_set.deep_copy(&source.storage.data_set);
                data_set
            },
            points: Some({
                let mut copy = Points::new();
                if let Some(points) = source.storage.points.as_ref() {
                    copy.deep_copy(points);
                }
                copy
            }),
            cell_locator: None,
            editable: source.storage.editable,
            modified_time,
        });
    }
}

impl DataSetApi for PointSet {
    fn data_set(&self) -> &DataSet {
        PointSet::data_set(self)
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        PointSet::data_set_mut(self)
    }

    fn get_class_name(&self) -> &'static str {
        "vtkPointSet"
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        PointSet::get_number_of_cells(self)
    }

    fn get_number_of_points(&self) -> VtkIdType {
        PointSet::get_number_of_points(self)
    }

    fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        PointSet::get_cell_type(self, cell_id)
    }

    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        PointSet::get_cell_points(self, cell_id, point_ids);
    }

    fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        PointSet::get_point(self, point_id)
    }
}
