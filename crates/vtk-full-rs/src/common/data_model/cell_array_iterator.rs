use crate::common::core::{IdList, Object, VtkIdType, VtkMTimeType};

use super::CellArray;

/// VTK: `vtkCellArrayIterator`.
#[derive(Debug)]
pub struct CellArrayIterator {
    object: Object,
    cell_array: *mut CellArray,
    temp_cell: IdList,
    current_cell_id: VtkIdType,
    number_of_cells: VtkIdType,
}

impl CellArrayIterator {
    /// VTK: `vtkCellArrayIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCellArrayIterator"),
            cell_array: std::ptr::null_mut(),
            temp_cell: IdList::new(),
            current_cell_id: 0,
            number_of_cells: 0,
        }
    }

    /// VTK protected macro: `SetCellArray`.
    pub(crate) fn set_cell_array(&mut self, cell_array: *mut CellArray) {
        self.cell_array = cell_array;
    }

    /// VTK: `vtkCellArrayIterator::GetCellArray`.
    pub fn get_cell_array(&self) -> *mut CellArray {
        self.cell_array
    }

    /// VTK: `vtkCellArrayIterator::GoToCell`.
    pub fn go_to_cell(&mut self, cell_id: VtkIdType) {
        self.current_cell_id = cell_id;
        self.number_of_cells = self.cell_array().get_number_of_cells();
        assert!(cell_id <= self.number_of_cells);
    }

    /// VTK: `vtkCellArrayIterator::GetCellAtId(vtkIdType)`.
    pub fn get_cell_at_id(&mut self, cell_id: VtkIdType) -> &IdList {
        self.go_to_cell(cell_id);
        self.get_current_cell()
    }

    /// VTK: `vtkCellArrayIterator::GetCellAtId(vtkIdType, vtkIdList*)`.
    pub fn get_cell_at_id_into_id_list(&mut self, cell_id: VtkIdType, cell_ids: &mut IdList) {
        self.go_to_cell(cell_id);
        self.get_current_cell_into_id_list(cell_ids);
    }

    /// VTK: `vtkCellArrayIterator::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType const*&)`.
    pub fn get_cell_at_id_tuple(&mut self, cell_id: VtkIdType) -> (VtkIdType, *const VtkIdType) {
        self.go_to_cell(cell_id);
        self.get_current_cell_tuple()
    }

    /// VTK: `vtkCellArrayIterator::GoToFirstCell`.
    pub fn go_to_first_cell(&mut self) {
        self.current_cell_id = 0;
        self.number_of_cells = self.cell_array().get_number_of_cells();
    }

    /// VTK: `vtkCellArrayIterator::GoToNextCell`.
    pub fn go_to_next_cell(&mut self) {
        self.current_cell_id += 1;
    }

    /// VTK: `vtkCellArrayIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> bool {
        self.current_cell_id >= self.number_of_cells
    }

    /// VTK: `vtkCellArrayIterator::GetCurrentCellId`.
    pub fn get_current_cell_id(&self) -> VtkIdType {
        self.current_cell_id
    }

    /// VTK: `vtkCellArrayIterator::GetCurrentCell()`.
    pub fn get_current_cell(&mut self) -> &IdList {
        assert!(self.current_cell_id < self.number_of_cells);
        let values = self
            .cell_array()
            .get_cell_at_id(self.current_cell_id)
            .to_vec();
        write_id_list(&mut self.temp_cell, &values);
        &self.temp_cell
    }

    /// VTK: `vtkCellArrayIterator::GetCurrentCell(vtkIdList*)`.
    pub fn get_current_cell_into_id_list(&self, ids: &mut IdList) {
        assert!(self.current_cell_id < self.number_of_cells);
        write_id_list(ids, self.cell_array().get_cell_at_id(self.current_cell_id));
    }

    /// VTK: `vtkCellArrayIterator::GetCurrentCell(vtkIdType&, vtkIdType const*&)`.
    pub fn get_current_cell_tuple(&mut self) -> (VtkIdType, *const VtkIdType) {
        assert!(self.current_cell_id < self.number_of_cells);
        if self.cell_array().is_storage_shareable() {
            let cell = self.cell_array().get_cell_at_id(self.current_cell_id);
            (cell.len() as VtkIdType, cell.as_ptr())
        } else {
            let values = self
                .cell_array()
                .get_cell_at_id(self.current_cell_id)
                .to_vec();
            write_id_list(&mut self.temp_cell, &values);
            (
                self.temp_cell.get_number_of_ids(),
                self.temp_cell.get_pointer(0).cast_const(),
            )
        }
    }

    /// VTK: `vtkCellArrayIterator::ReplaceCurrentCell(vtkIdList*)`.
    pub fn replace_current_cell(&mut self, list: &IdList) {
        assert!(self.current_cell_id < self.number_of_cells);
        let current_cell_id = self.current_cell_id;
        let point_ids = id_list_values(list);
        self.cell_array_mut()
            .replace_cell_at_id(current_cell_id, &point_ids);
    }

    /// VTK: `vtkCellArrayIterator::ReplaceCurrentCell(vtkIdType, const vtkIdType*)`.
    pub fn replace_current_cell_with_slice(&mut self, npts: VtkIdType, pts: &[VtkIdType]) {
        assert!(self.current_cell_id < self.number_of_cells);
        let npts = usize::try_from(npts).expect("cell size must be non-negative");
        assert_eq!(npts, pts.len());
        let current_cell_id = self.current_cell_id;
        self.cell_array_mut()
            .replace_cell_at_id(current_cell_id, pts);
    }

    /// VTK: `vtkCellArrayIterator::ReverseCurrentCell`.
    pub fn reverse_current_cell(&mut self) {
        assert!(self.current_cell_id < self.number_of_cells);
        let current_cell_id = self.current_cell_id;
        self.cell_array_mut().reverse_cell_at_id(current_cell_id);
    }

    /// VTK: `vtkCellArrayIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkCellArrayIterator {{ CurrentCellId: {}, CellArray: {:?} }}",
            self.current_cell_id, self.cell_array
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCellArrayIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCellArrayIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkCellArrayIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCellArrayIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkCellArrayIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCellArrayIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    fn cell_array(&self) -> &CellArray {
        assert!(!self.cell_array.is_null(), "CellArray is null");
        unsafe { &*self.cell_array }
    }

    fn cell_array_mut(&mut self) -> &mut CellArray {
        assert!(!self.cell_array.is_null(), "CellArray is null");
        unsafe { &mut *self.cell_array }
    }
}

impl Default for CellArrayIterator {
    fn default() -> Self {
        Self::new()
    }
}

fn write_id_list(ids: &mut IdList, values: &[VtkIdType]) {
    ids.reset();
    ids.set_number_of_ids(values.len() as VtkIdType);
    for (idx, value) in values.iter().enumerate() {
        ids.set_id(idx as VtkIdType, *value);
    }
}

fn id_list_values(ids: &IdList) -> Vec<VtkIdType> {
    (0..ids.get_number_of_ids())
        .map(|idx| ids.get_id(idx))
        .collect()
}
