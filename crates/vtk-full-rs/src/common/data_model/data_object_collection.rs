use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};

use super::DataObject;

/// VTK: `vtkDataObjectCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct DataObjectCollection {
    object: Object,
    current: usize,
    data_objects: Vec<NonNull<DataObject>>,
}

impl DataObjectCollection {
    /// VTK: `vtkDataObjectCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataObjectCollection"),
            current: 0,
            data_objects: Vec::new(),
        }
    }

    /// VTK: `vtkDataObjectCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkDataObjectCollection::AddItem(vtkDataObject*)`.
    pub fn add_item(&mut self, data_object: *mut DataObject) {
        let Some(data_object) = NonNull::new(data_object) else {
            return;
        };
        self.data_objects.push(data_object);
        self.modified();
    }

    /// VTK: `vtkDataObjectCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut DataObject {
        if self.current >= self.data_objects.len() {
            return ptr::null_mut();
        }
        let data_object = self.data_objects[self.current].as_ptr();
        self.current += 1;
        data_object
    }

    /// VTK: `vtkDataObjectCollection::GetItem`.
    pub fn get_item(&self, i: i32) -> *mut DataObject {
        if i < 0 {
            return ptr::null_mut();
        }
        self.data_objects
            .get(i as usize)
            .map_or(ptr::null_mut(), |data_object| data_object.as_ptr())
    }

    /// VTK: `vtkDataObjectCollection::GetNextDataObject(vtkCollectionSimpleIterator&)`.
    pub fn get_next_data_object(&self, cookie: &mut CollectionSimpleIterator) -> *mut DataObject {
        if *cookie >= self.data_objects.len() {
            return ptr::null_mut();
        }
        let data_object = self.data_objects[*cookie].as_ptr();
        *cookie += 1;
        data_object
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.data_objects.len() as i32
    }

    /// VTK: `vtkCollection::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.current = 0;
    }

    /// VTK: `vtkCollection::InitTraversal(vtkCollectionSimpleIterator&)`.
    pub fn init_traversal_cookie(&self, cookie: &mut CollectionSimpleIterator) {
        *cookie = 0;
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
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

impl Default for DataObjectCollection {
    fn default() -> Self {
        Self::new()
    }
}
