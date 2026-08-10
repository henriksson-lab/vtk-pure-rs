use std::ffi::c_void;

use crate::common::core::{Object, ObjectBaseApi, VtkIdType, VtkMTimeType};

/// VTK: `vtkPointCentered`.
pub const VTK_POINT_CENTERED: i32 = 0;
/// VTK: `vtkCellCentered`.
pub const VTK_CELL_CENTERED: i32 = 1;
/// VTK: `vtkBoundaryCentered`.
pub const VTK_BOUNDARY_CENTERED: i32 = 2;

/// VTK: `vtkGenericAdaptorCell*`.
pub type GenericAdaptorCellHandle = *mut c_void;
/// VTK: `vtkGenericCellIterator*`.
pub type GenericCellIteratorHandle = *mut c_void;
/// VTK: `vtkGenericPointIterator*`.
pub type GenericPointIteratorHandle = *mut c_void;

/// VTK: `vtkGenericAttribute`.
///
/// This stores the abstract VTK base-class identity. Concrete adaptor-framework
/// attributes implement `GenericAttributeApi`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericAttribute {
    object: Object,
}

impl GenericAttribute {
    /// VTK: `vtkGenericAttribute::vtkGenericAttribute`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
        }
    }

    /// VTK: `vtkGenericAttribute::PrintSelf`.
    pub fn print_self<T: GenericAttributeApi + ?Sized>(&self, attribute: &T) -> String {
        let centering = match attribute.get_centering() {
            VTK_POINT_CENTERED => "on points",
            VTK_CELL_CENTERED => "on cells",
            VTK_BOUNDARY_CENTERED => "on boundaries",
            _ => "unknown",
        };
        format!(
            "{}Name: {}\nNumber of components: {}\nCentering: {}\n",
            self.object.print_self(),
            attribute.get_name().unwrap_or("(null)"),
            attribute.get_number_of_components(),
            centering
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkGenericAttribute::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkGenericAttribute" || Object::is_type_of(name)
    }

    /// VTK: `vtkGenericAttribute::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkGenericAttribute::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkGenericAttribute" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkGenericAttribute::GetNumberOfGenerationsFromBase`.
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

impl Default for GenericAttribute {
    fn default() -> Self {
        Self::with_class_name("vtkGenericAttribute")
    }
}

/// VTK pure virtual API for `vtkGenericAttribute`.
pub trait GenericAttributeApi {
    /// Access to the translated abstract base state.
    fn generic_attribute(&self) -> &GenericAttribute;

    /// Mutable access to the translated abstract base state.
    fn generic_attribute_mut(&mut self) -> &mut GenericAttribute;

    /// VTK: `vtkGenericAttribute::GetName`.
    fn get_name(&self) -> Option<&str>;

    /// VTK: `vtkGenericAttribute::GetNumberOfComponents`.
    fn get_number_of_components(&self) -> i32;

    /// VTK: `vtkGenericAttribute::GetCentering`.
    fn get_centering(&self) -> i32;

    /// VTK: `vtkGenericAttribute::GetType`.
    fn get_type(&self) -> i32;

    /// VTK: `vtkGenericAttribute::GetComponentType`.
    fn get_component_type(&self) -> i32;

    /// VTK: `vtkGenericAttribute::GetSize`.
    fn get_size(&self) -> VtkIdType;

    /// VTK: `vtkGenericAttribute::GetActualMemorySize`.
    fn get_actual_memory_size(&self) -> u64;

    /// VTK: `vtkGenericAttribute::GetRange(int)`.
    fn get_range(&mut self, component: i32) -> Option<&[f64; 2]>;

    /// VTK: `vtkGenericAttribute::GetRange(int, double[2])`.
    fn get_range_into(&self, component: i32, range: &mut [f64; 2]);

    /// VTK: `vtkGenericAttribute::GetMaxNorm`.
    fn get_max_norm(&self) -> f64;

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericAdaptorCell*)`.
    fn get_tuple_adaptor_cell(&mut self, cell: GenericAdaptorCellHandle) -> Option<&[f64]>;

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericAdaptorCell*, double*)`.
    fn get_tuple_adaptor_cell_into(&self, cell: GenericAdaptorCellHandle, tuple: &mut [f64]);

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericCellIterator*)`.
    fn get_tuple_cell_iterator(&mut self, cell: GenericCellIteratorHandle) -> Option<&[f64]>;

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericCellIterator*, double*)`.
    fn get_tuple_cell_iterator_into(&self, cell: GenericCellIteratorHandle, tuple: &mut [f64]);

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericPointIterator*)`.
    fn get_tuple_point_iterator(&mut self, point: GenericPointIteratorHandle) -> Option<&[f64]>;

    /// VTK: `vtkGenericAttribute::GetTuple(vtkGenericPointIterator*, double*)`.
    fn get_tuple_point_iterator_into(&self, point: GenericPointIteratorHandle, tuple: &mut [f64]);

    /// VTK: `vtkGenericAttribute::GetComponent(int, vtkGenericCellIterator*, double*)`.
    fn get_component_cell_iterator(
        &self,
        component: i32,
        cell: GenericCellIteratorHandle,
        values: &mut [f64],
    );

    /// VTK: `vtkGenericAttribute::GetComponent(int, vtkGenericPointIterator*)`.
    fn get_component_point_iterator(
        &self,
        component: i32,
        point: GenericPointIteratorHandle,
    ) -> f64;

    /// VTK: `vtkGenericAttribute::DeepCopy`.
    fn deep_copy(&mut self, other: &dyn GenericAttributeApi);

    /// VTK: `vtkGenericAttribute::ShallowCopy`.
    fn shallow_copy(&mut self, other: &dyn GenericAttributeApi);

    /// VTK: `vtkGenericAttribute::PrintSelf`.
    fn print_self(&self) -> String {
        self.generic_attribute().print_self(self)
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str {
        self.generic_attribute().get_class_name()
    }

    /// VTK: `vtkGenericAttribute::IsA`.
    fn is_a(&self, name: &str) -> bool {
        self.generic_attribute().is_a(name)
    }

    /// VTK: `vtkGenericAttribute::GetNumberOfGenerationsFromBase`.
    fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        self.generic_attribute()
            .get_number_of_generations_from_base(name)
    }

    /// VTK: `vtkObject::Modified`.
    fn modified(&mut self) {
        self.generic_attribute_mut().modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType {
        self.generic_attribute().get_m_time()
    }
}
