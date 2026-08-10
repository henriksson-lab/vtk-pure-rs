use std::ptr::{self, NonNull};

use super::{
    any_array::AnyArray,
    data_array_collection::DataArrayCollection,
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkDataArrayCollectionIterator`.
#[derive(Debug)]
pub struct DataArrayCollectionIterator {
    object: Object,
    collection: Option<NonNull<DataArrayCollection>>,
    iterator: usize,
}

impl DataArrayCollectionIterator {
    /// VTK: `vtkDataArrayCollectionIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataArrayCollectionIterator"),
            collection: None,
            iterator: 0,
        }
    }

    /// VTK: `vtkDataArrayCollectionIterator::SetCollection(vtkDataArrayCollection*)`.
    ///
    /// # Safety
    ///
    /// `collection` must be null or point to a live `DataArrayCollection` that
    /// remains valid while this iterator references it.
    pub unsafe fn set_collection(&mut self, collection: *mut DataArrayCollection) {
        self.collection = NonNull::new(collection);
        self.modified();
        self.go_to_first_item();
    }

    /// VTK: `vtkCollectionIterator::GetCollection`.
    pub fn get_collection(&self) -> *mut DataArrayCollection {
        self.collection
            .map_or(ptr::null_mut(), |collection| collection.as_ptr())
    }

    /// VTK: `vtkCollectionIterator::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.go_to_first_item();
    }

    /// VTK: `vtkCollectionIterator::GoToFirstItem`.
    pub fn go_to_first_item(&mut self) {
        if self.collection.is_some() {
            self.iterator = 0;
        }
    }

    /// VTK: `vtkCollectionIterator::GoToNextItem`.
    pub fn go_to_next_item(&mut self) {
        if let Some(collection) = self.collection {
            let collection = unsafe { collection.as_ref() };
            if self.iterator < collection.len() {
                self.iterator += 1;
            }
        }
    }

    /// VTK: `vtkCollectionIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> i32 {
        let Some(collection) = self.collection else {
            return 1;
        };
        let collection = unsafe { collection.as_ref() };
        i32::from(self.iterator >= collection.len())
    }

    /// VTK: `vtkDataArrayCollectionIterator::GetDataArray`.
    pub fn get_data_array(&self) -> *mut AnyArray {
        let Some(collection) = self.collection else {
            return ptr::null_mut();
        };
        let collection = unsafe { collection.as_ref() };
        if self.iterator < collection.len() {
            collection.data_array_at_raw(self.iterator)
        } else {
            ptr::null_mut()
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkDataArrayCollectionIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkDataArrayCollectionIterator"
            || name == "vtkCollectionIterator"
            || Object::is_type_of(name)
    }

    /// VTK: `vtkDataArrayCollectionIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkDataArrayCollectionIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkDataArrayCollectionIterator" => 0,
            "vtkCollectionIterator" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkDataArrayCollectionIterator::GetNumberOfGenerationsFromBase`.
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

impl Default for DataArrayCollectionIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DataArrayCollectionIterator {
    fn drop(&mut self) {
        self.collection = None;
    }
}
