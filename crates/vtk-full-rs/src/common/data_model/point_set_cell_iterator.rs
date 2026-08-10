use crate::common::core::{IdList, Points, VtkIdType, VtkMTimeType};

use super::{CellIterator, CellIteratorApi, PointSet};

/// VTK: `vtkPointSetCellIterator`.
#[derive(Debug, Clone)]
pub struct PointSetCellIterator {
    cell_iterator: CellIterator,
    point_set: *mut PointSet,
    point_set_points: Option<Points>,
    cell_id: VtkIdType,
}

impl PointSetCellIterator {
    /// VTK: `vtkPointSetCellIterator::New`.
    pub fn new() -> Self {
        Self {
            cell_iterator: CellIterator::with_class_name("vtkPointSetCellIterator"),
            point_set: std::ptr::null_mut(),
            point_set_points: None,
            cell_id: 0,
        }
    }

    /// VTK protected friend method: `vtkPointSetCellIterator::SetPointSet`.
    pub(crate) fn set_point_set(&mut self, point_set: *mut PointSet) {
        self.point_set = point_set;
        self.point_set_points = self.point_set().and_then(PointSet::get_points).cloned();
        self.cell_id = 0;
        if let Some(points) = self.point_set_points.as_ref() {
            self.cell_iterator
                .points_mut()
                .set_data_type(points.get_data_type());
        }
    }

    /// VTK: `vtkPointSetCellIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}PointSet: {:?}\n",
            self.cell_iterator.print_self(),
            self.point_set
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell_iterator.get_class_name()
    }

    /// VTK: `vtkPointSetCellIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkPointSetCellIterator" || CellIterator::is_type_of(name)
    }

    /// VTK: `vtkPointSetCellIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkPointSetCellIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkPointSetCellIterator" => 0,
            "vtkCellIterator" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => CellIterator::get_number_of_generations_from_base_type(name) + 1,
        }
    }

    /// VTK: `vtkPointSetCellIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.cell_iterator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.cell_iterator.get_m_time()
    }

    /// VTK: `vtkCellIterator::InitTraversal`.
    pub fn init_traversal(&mut self) {
        <Self as CellIteratorApi>::init_traversal(self);
    }

    /// VTK: `vtkCellIterator::GoToNextCell`.
    pub fn go_to_next_cell(&mut self) {
        <Self as CellIteratorApi>::go_to_next_cell(self);
    }

    /// VTK: `vtkPointSetCellIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> bool {
        <Self as CellIteratorApi>::is_done_with_traversal(self)
    }

    /// VTK: `vtkPointSetCellIterator::GetCellId`.
    pub fn get_cell_id(&self) -> VtkIdType {
        <Self as CellIteratorApi>::get_cell_id(self)
    }

    /// VTK: `vtkCellIterator::GetCellType`.
    pub fn get_cell_type(&mut self) -> i32 {
        <Self as CellIteratorApi>::get_cell_type(self)
    }

    /// VTK: `vtkCellIterator::GetCellDimension`.
    pub fn get_cell_dimension(&mut self) -> i32 {
        <Self as CellIteratorApi>::get_cell_dimension(self)
    }

    /// VTK: `vtkCellIterator::GetPointIds`.
    pub fn get_point_ids(&mut self) -> &IdList {
        <Self as CellIteratorApi>::get_point_ids(self)
    }

    /// VTK: `vtkCellIterator::GetPoints`.
    pub fn get_points(&mut self) -> &Points {
        <Self as CellIteratorApi>::get_points(self)
    }

    /// VTK: `vtkCellIterator::GetCellFaces`.
    pub fn get_cell_faces(&mut self) -> &super::CellArray {
        <Self as CellIteratorApi>::get_cell_faces(self)
    }

    /// VTK: `vtkCellIterator::GetSerializedCellFaces`.
    pub fn get_serialized_cell_faces(&mut self) -> &IdList {
        <Self as CellIteratorApi>::get_serialized_cell_faces(self)
    }

    /// VTK: `vtkCellIterator::GetNumberOfPoints`.
    pub fn get_number_of_points(&mut self) -> VtkIdType {
        <Self as CellIteratorApi>::get_number_of_points(self)
    }

    /// VTK: `vtkCellIterator::GetNumberOfFaces`.
    pub fn get_number_of_faces(&mut self) -> VtkIdType {
        <Self as CellIteratorApi>::get_number_of_faces(self)
    }

    fn point_set(&self) -> Option<&PointSet> {
        if self.point_set.is_null() {
            None
        } else {
            Some(unsafe { &*self.point_set })
        }
    }
}

impl CellIteratorApi for PointSetCellIterator {
    fn cell_iterator(&self) -> &CellIterator {
        &self.cell_iterator
    }

    fn cell_iterator_mut(&mut self) -> &mut CellIterator {
        &mut self.cell_iterator
    }

    fn is_done_with_traversal(&self) -> bool {
        self.point_set()
            .is_none_or(|point_set| self.cell_id >= point_set.get_number_of_cells())
    }

    fn get_cell_id(&self) -> VtkIdType {
        self.cell_id
    }

    fn reset_to_first_cell(&mut self) {
        self.cell_id = 0;
    }

    fn increment_to_next_cell(&mut self) {
        self.cell_id += 1;
    }

    fn fetch_cell_type(&mut self) {
        let cell_type = self
            .point_set()
            .expect("vtkPointSetCellIterator PointSet must be set")
            .get_cell_type(self.cell_id);
        self.cell_iterator.set_cell_type(cell_type);
    }

    fn fetch_point_ids(&mut self) {
        let mut point_ids = IdList::new();
        self.point_set()
            .expect("vtkPointSetCellIterator PointSet must be set")
            .get_cell_points(self.cell_id, &mut point_ids);
        self.cell_iterator.point_ids_mut().deep_copy(&point_ids);
    }

    fn fetch_points(&mut self) {
        let point_ids: Vec<_> = self.get_point_ids().iter().collect();
        self.point_set_points
            .as_ref()
            .expect("vtkPointSetCellIterator PointSet points must be set")
            .get_points(&point_ids, self.cell_iterator.points_mut());
    }
}

impl Default for PointSetCellIterator {
    fn default() -> Self {
        Self::new()
    }
}
