use crate::common::core::{IdTypeArray, Object, UnsignedCharArray, VtkIdType, VtkMTimeType};

use super::cell_type_utilities::CellTypeUtilities;

const DEFAULT_ALLOCATE_SIZE: VtkIdType = 512;
const VTK_EMPTY_CELL: u8 = 0;

/// VTK: `vtkCellTypes`.
#[derive(Debug, Clone, PartialEq)]
pub struct CellTypes {
    object: Object,
    type_array: UnsignedCharArray,
    location_array: IdTypeArray,
    max_id: VtkIdType,
}

impl CellTypes {
    /// VTK: `vtkCellTypes::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCellTypes"),
            type_array: UnsignedCharArray::new(),
            location_array: IdTypeArray::new(),
            max_id: -1,
        }
    }

    /// VTK: `vtkCellTypes::Allocate`.
    pub fn allocate(&mut self, sz: VtkIdType, _ext: VtkIdType) -> i32 {
        self.max_id = -1;
        self.type_array.initialize();
        self.type_array.reserve_values(sz);
        1
    }

    /// Rust default-argument helper for VTK `vtkCellTypes::Allocate`.
    pub fn allocate_default(&mut self) -> i32 {
        self.allocate(DEFAULT_ALLOCATE_SIZE, 1000)
    }

    /// VTK: `vtkCellTypes::InsertCell(vtkIdType, unsigned char)`.
    pub fn insert_cell(&mut self, cell_id: VtkIdType, cell_type: u8) {
        self.type_array.insert_typed_tuple(cell_id, &[cell_type]);
        self.max_id = self.max_id.max(cell_id);
    }

    /// VTK: deprecated `vtkCellTypes::InsertCell(vtkIdType, unsigned char, vtkIdType)`.
    pub fn insert_cell_with_location(
        &mut self,
        cell_id: VtkIdType,
        cell_type: u8,
        _location: VtkIdType,
    ) {
        self.insert_cell(cell_id, cell_type);
    }

    /// VTK: `vtkCellTypes::InsertNextCell(unsigned char)`.
    pub fn insert_next_cell(&mut self, cell_type: u8) -> VtkIdType {
        let cell_id = self.max_id + 1;
        self.insert_cell(cell_id, cell_type);
        self.max_id
    }

    /// VTK: deprecated `vtkCellTypes::InsertNextCell(unsigned char, vtkIdType)`.
    pub fn insert_next_cell_with_location(
        &mut self,
        cell_type: u8,
        _location: VtkIdType,
    ) -> VtkIdType {
        self.insert_next_cell(cell_type)
    }

    /// VTK: `vtkCellTypes::SetCellTypes`.
    pub fn set_cell_types(&mut self, ncells: VtkIdType, cell_types: &UnsignedCharArray) {
        self.type_array.shallow_copy(cell_types);
        self.max_id = ncells - 1;
    }

    /// VTK: `vtkCellTypes::DeleteCell`.
    pub fn delete_cell(&mut self, cell_id: VtkIdType) {
        self.type_array.set_typed_tuple(cell_id, &[VTK_EMPTY_CELL]);
    }

    /// VTK: `vtkCellTypes::GetNumberOfTypes`.
    pub fn get_number_of_types(&self) -> VtkIdType {
        self.max_id + 1
    }

    /// VTK: `vtkCellTypes::IsType`.
    pub fn is_type(&self, cell_type: u8) -> i32 {
        for cell_id in 0..self.get_number_of_types() {
            if self.get_cell_type(cell_id) == cell_type {
                return 1;
            }
        }
        0
    }

    /// VTK: `vtkCellTypes::InsertNextType`.
    pub fn insert_next_type(&mut self, cell_type: u8) -> VtkIdType {
        self.insert_next_cell(cell_type)
    }

    /// VTK: `vtkCellTypes::GetCellType`.
    pub fn get_cell_type(&self, cell_id: VtkIdType) -> u8 {
        self.type_array.get_typed_tuple(cell_id)[0]
    }

    /// VTK: `vtkCellTypes::Squeeze`.
    pub fn squeeze(&mut self) {
        self.type_array.squeeze();
    }

    /// VTK: `vtkCellTypes::Reset`.
    pub fn reset(&mut self) {
        self.max_id = -1;
    }

    /// VTK: `vtkCellTypes::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.type_array.get_actual_memory_size().div_ceil(1024)
    }

    /// VTK: `vtkCellTypes::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.type_array.deep_copy(&source.type_array);
        self.max_id = source.max_id;
    }

    /// VTK: deprecated `vtkCellTypes::GetClassNameFromTypeId`.
    pub fn get_class_name_from_type_id(type_id: i32) -> &'static str {
        CellTypeUtilities::get_class_name_from_type_id(type_id)
    }

    /// VTK: deprecated `vtkCellTypes::GetTypeIdFromClassName`.
    pub fn get_type_id_from_class_name(classname: Option<&str>) -> i32 {
        CellTypeUtilities::get_type_id_from_class_name(classname)
    }

    /// VTK: deprecated `vtkCellTypes::IsLinear`.
    pub fn is_linear(cell_type: u8) -> i32 {
        i32::from(CellTypeUtilities::is_linear(cell_type))
    }

    /// VTK: deprecated `vtkCellTypes::GetDimension`.
    pub fn get_dimension(cell_type: u8) -> i32 {
        CellTypeUtilities::get_dimension(cell_type)
    }

    /// VTK: `vtkCellTypes::GetCellTypesArray`.
    pub fn get_cell_types_array(&self) -> &UnsignedCharArray {
        &self.type_array
    }

    /// Mutable Rust boundary for VTK `vtkCellTypes::GetCellTypesArray`.
    pub fn get_cell_types_array_mut(&mut self) -> &mut UnsignedCharArray {
        &mut self.type_array
    }

    /// VTK: deprecated `vtkCellTypes::GetCellLocationsArray`.
    pub fn get_cell_locations_array(&self) -> &IdTypeArray {
        &self.location_array
    }

    /// Mutable Rust boundary for VTK `vtkCellTypes::GetCellLocationsArray`.
    pub fn get_cell_locations_array_mut(&mut self) -> &mut IdTypeArray {
        &mut self.location_array
    }

    /// VTK: `vtkCellTypes::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkCellTypes {{ TypeArray: {:?}, MaxId: {} }}",
            self.type_array, self.max_id
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCellTypes::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCellTypes" || Object::is_type_of(name)
    }

    /// VTK: `vtkCellTypes::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCellTypes::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkCellTypes" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCellTypes::GetNumberOfGenerationsFromBase`.
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
}

impl Default for CellTypes {
    fn default() -> Self {
        Self::new()
    }
}
