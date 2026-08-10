use crate::common::core::{Object, ObjectBaseApi, VtkIdType, VtkMTimeType};

use super::FieldData;

/// VTK: `vtkSortFieldData`.
///
/// The translated class preserves the static field-data sort entry points.
/// Inherited `vtkSortDataArray` instance methods are deferred until that
/// superclass is translated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortFieldData {
    object: Object,
}

impl SortFieldData {
    /// VTK: `vtkSortFieldData::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSortFieldData"),
        }
    }

    /// VTK: inline `vtkSortFieldData::Sort(vtkFieldData*, const char*, int, int)`.
    pub fn sort(
        fd: Option<&mut FieldData>,
        array_name: Option<&str>,
        k: i32,
        return_indices: i32,
    ) -> Option<Vec<VtkIdType>> {
        Self::sort_with_direction(fd, array_name, k, return_indices, 0)
    }

    /// VTK: `vtkSortFieldData::Sort(vtkFieldData*, const char*, int, int, int)`.
    pub fn sort_with_direction(
        fd: Option<&mut FieldData>,
        array_name: Option<&str>,
        k: i32,
        return_indices: i32,
        dir: i32,
    ) -> Option<Vec<VtkIdType>> {
        let fd = fd?;
        let array_name = array_name?;
        let idx = fd.sort_tuples_by_component(array_name, k)?;
        let number_of_tuples = idx.len();
        fd.shuffle_arrays_with_tuple_count(number_of_tuples, &idx, dir);
        (return_indices != 0).then_some(idx)
    }

    /// VTK: `vtkSortFieldData::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkSortFieldData::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkSortFieldData" || Object::is_type_of(name)
    }

    /// VTK: `vtkSortFieldData::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkSortFieldData::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkSortFieldData" => 0,
            "vtkSortDataArray" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkSortFieldData::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
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

impl Default for SortFieldData {
    fn default() -> Self {
        Self::new()
    }
}
