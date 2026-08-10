use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, ObjectFactoryHandle, VtkMTimeType};

/// VTK: `vtkObjectFactoryCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectFactoryCollection {
    object: Object,
    current: usize,
    object_factories: Vec<NonNull<std::ffi::c_void>>,
}

impl ObjectFactoryCollection {
    /// VTK: `vtkObjectFactoryCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkObjectFactoryCollection"),
            current: 0,
            object_factories: Vec::new(),
        }
    }

    /// VTK: `vtkObjectFactoryCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Number Of Items: {}\n", self.object_factories.len())
    }

    /// VTK: `vtkObjectFactoryCollection::AddItem`.
    pub fn add_item(&mut self, factory: ObjectFactoryHandle) {
        let Some(factory) = NonNull::new(factory) else {
            return;
        };
        self.object_factories.push(factory);
        self.modified();
    }

    /// VTK: `vtkObjectFactoryCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> ObjectFactoryHandle {
        if self.current >= self.object_factories.len() {
            return ptr::null_mut();
        }
        let factory = self.object_factories[self.current].as_ptr();
        self.current += 1;
        factory
    }

    /// VTK: `vtkObjectFactoryCollection::GetNextObjectFactory`.
    pub fn get_next_object_factory(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> ObjectFactoryHandle {
        if *cookie >= self.object_factories.len() {
            return ptr::null_mut();
        }
        let factory = self.object_factories[*cookie].as_ptr();
        *cookie += 1;
        factory
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
        self.object_factories.len() as i32
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

impl Default for ObjectFactoryCollection {
    fn default() -> Self {
        Self::new()
    }
}
