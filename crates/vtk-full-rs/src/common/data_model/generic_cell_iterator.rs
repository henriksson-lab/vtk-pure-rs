use crate::common::core::{Object, ObjectBaseApi, VtkMTimeType};

use super::generic_attribute::GenericAdaptorCellHandle;

/// VTK: `vtkGenericCellIterator`.
///
/// This stores the abstract VTK base-class identity. Concrete adaptor-framework
/// cell iterators implement `GenericCellIteratorApi`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericCellIterator {
    object: Object,
}

impl GenericCellIterator {
    /// VTK: `vtkGenericCellIterator::vtkGenericCellIterator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
        }
    }

    /// VTK: `vtkGenericCellIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkGenericCellIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkGenericCellIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkGenericCellIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkGenericCellIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkGenericCellIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkGenericCellIterator::GetNumberOfGenerationsFromBase`.
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

impl Default for GenericCellIterator {
    fn default() -> Self {
        Self::with_class_name("vtkGenericCellIterator")
    }
}

/// VTK pure virtual API for `vtkGenericCellIterator`.
pub trait GenericCellIteratorApi {
    /// Access to the translated abstract base state.
    fn generic_cell_iterator(&self) -> &GenericCellIterator;

    /// Mutable access to the translated abstract base state.
    fn generic_cell_iterator_mut(&mut self) -> &mut GenericCellIterator;

    /// VTK: `vtkGenericCellIterator::Begin`.
    fn begin(&mut self);

    /// VTK: `vtkGenericCellIterator::IsAtEnd`.
    fn is_at_end(&self) -> bool;

    /// VTK: `vtkGenericCellIterator::NewCell`.
    fn new_cell(&self) -> GenericAdaptorCellHandle;

    /// VTK: `vtkGenericCellIterator::GetCell(vtkGenericAdaptorCell*)`.
    fn get_cell_into(&self, cell: GenericAdaptorCellHandle);

    /// VTK: `vtkGenericCellIterator::GetCell()`.
    fn get_cell(&mut self) -> GenericAdaptorCellHandle;

    /// VTK: `vtkGenericCellIterator::Next`.
    fn next(&mut self);

    /// VTK: `vtkGenericCellIterator::PrintSelf`.
    fn print_self(&self) -> String {
        self.generic_cell_iterator().print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str {
        self.generic_cell_iterator().get_class_name()
    }

    /// VTK: `vtkGenericCellIterator::IsA`.
    fn is_a(&self, name: &str) -> bool {
        self.generic_cell_iterator().is_a(name)
    }

    /// VTK: `vtkGenericCellIterator::GetNumberOfGenerationsFromBase`.
    fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        self.generic_cell_iterator()
            .get_number_of_generations_from_base(name)
    }

    /// VTK: `vtkObject::Modified`.
    fn modified(&mut self) {
        self.generic_cell_iterator_mut().modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType {
        self.generic_cell_iterator().get_m_time()
    }
}
