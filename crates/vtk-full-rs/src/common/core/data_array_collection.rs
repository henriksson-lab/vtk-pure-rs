use std::ptr::{self, NonNull};

use super::{
    any_array::AnyArray,
    collection::CollectionSimpleIterator,
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkDataArrayCollection`.
#[derive(Debug)]
pub struct DataArrayCollection {
    object: Object,
    current: usize,
    arrays: Vec<NonNull<AnyArray>>,
}

impl DataArrayCollection {
    /// VTK: `vtkDataArrayCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataArrayCollection"),
            current: 0,
            arrays: Vec::new(),
        }
    }

    /// VTK: `vtkDataArrayCollection::AddItem`.
    ///
    /// # Safety
    ///
    /// `array` must point to a live numeric `AnyArray` that remains valid while
    /// it is stored in this collection.
    pub unsafe fn add_item(&mut self, array: *mut AnyArray) {
        let array =
            NonNull::new(array).expect("vtkDataArrayCollection::AddItem array must not be null");
        assert!(
            unsafe { array.as_ref() }.is_data_array(),
            "vtkDataArrayCollection accepts vtkDataArray instances only"
        );
        self.arrays.push(array);
        self.modified();
    }

    /// VTK: `vtkDataArrayCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut AnyArray {
        if self.current >= self.arrays.len() {
            return ptr::null_mut();
        }
        let array = self.arrays[self.current].as_ptr();
        self.current += 1;
        array
    }

    /// VTK: `vtkDataArrayCollection::GetItem`.
    pub fn get_item(&self, i: i32) -> *mut AnyArray {
        if i < 0 {
            return ptr::null_mut();
        }
        self.arrays
            .get(i as usize)
            .map_or(ptr::null_mut(), |array| array.as_ptr())
    }

    /// VTK: `vtkDataArrayCollection::GetNextDataArray`.
    pub fn get_next_data_array(&self, cookie: &mut CollectionSimpleIterator) -> *mut AnyArray {
        if *cookie >= self.arrays.len() {
            return ptr::null_mut();
        }
        let array = self.arrays[*cookie].as_ptr();
        *cookie += 1;
        array
    }

    /// VTK: `vtkCollection::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.current = 0;
    }

    /// VTK: `vtkCollection::InitTraversal(vtkCollectionSimpleIterator&)`.
    pub fn init_traversal_cookie(&self, cookie: &mut CollectionSimpleIterator) {
        *cookie = 0;
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.arrays.len() as i32
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkDataArrayCollection::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkDataArrayCollection" || name == "vtkCollection" || Object::is_type_of(name)
    }

    /// VTK: `vtkDataArrayCollection::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkDataArrayCollection::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkDataArrayCollection" => 0,
            "vtkCollection" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkDataArrayCollection::GetNumberOfGenerationsFromBase`.
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

    pub(crate) fn len(&self) -> usize {
        self.arrays.len()
    }

    pub(crate) fn data_array_at_raw(&self, index: usize) -> *mut AnyArray {
        self.arrays
            .get(index)
            .map_or(ptr::null_mut(), |array| array.as_ptr())
    }
}

impl Default for DataArrayCollection {
    fn default() -> Self {
        Self::new()
    }
}
