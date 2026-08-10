use crate::common::core::{Object, ObjectBaseApi, VtkIdType, VtkMTimeType};

/// VTK: `vtkGenericPointIterator`.
///
/// This stores the abstract VTK base-class identity. Concrete adaptor-framework
/// point iterators implement `GenericPointIteratorApi`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericPointIterator {
    object: Object,
}

impl GenericPointIterator {
    /// VTK: `vtkGenericPointIterator::vtkGenericPointIterator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
        }
    }

    /// VTK: `vtkGenericPointIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkGenericPointIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkGenericPointIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkGenericPointIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkGenericPointIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkGenericPointIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkGenericPointIterator::GetNumberOfGenerationsFromBase`.
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

impl Default for GenericPointIterator {
    fn default() -> Self {
        Self::with_class_name("vtkGenericPointIterator")
    }
}

/// VTK pure virtual API for `vtkGenericPointIterator`.
pub trait GenericPointIteratorApi {
    /// Access to the translated abstract base state.
    fn generic_point_iterator(&self) -> &GenericPointIterator;

    /// Mutable access to the translated abstract base state.
    fn generic_point_iterator_mut(&mut self) -> &mut GenericPointIterator;

    /// VTK: `vtkGenericPointIterator::Begin`.
    fn begin(&mut self);

    /// VTK: `vtkGenericPointIterator::IsAtEnd`.
    fn is_at_end(&self) -> bool;

    /// VTK: `vtkGenericPointIterator::Next`.
    fn next(&mut self);

    /// VTK: `vtkGenericPointIterator::GetPosition()`.
    fn get_position(&mut self) -> Option<&[f64; 3]>;

    /// VTK: `vtkGenericPointIterator::GetPosition(double[3])`.
    fn get_position_into(&self, x: &mut [f64; 3]);

    /// VTK: `vtkGenericPointIterator::GetId`.
    fn get_id(&self) -> VtkIdType;

    /// VTK: `vtkGenericPointIterator::PrintSelf`.
    fn print_self(&self) -> String {
        self.generic_point_iterator().print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str {
        self.generic_point_iterator().get_class_name()
    }

    /// VTK: `vtkGenericPointIterator::IsA`.
    fn is_a(&self, name: &str) -> bool {
        self.generic_point_iterator().is_a(name)
    }

    /// VTK: `vtkGenericPointIterator::GetNumberOfGenerationsFromBase`.
    fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        self.generic_point_iterator()
            .get_number_of_generations_from_base(name)
    }

    /// VTK: `vtkObject::Modified`.
    fn modified(&mut self) {
        self.generic_point_iterator_mut().modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType {
        self.generic_point_iterator().get_m_time()
    }
}
