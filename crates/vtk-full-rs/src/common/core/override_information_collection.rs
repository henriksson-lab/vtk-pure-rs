use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, OverrideInformation, VtkMTimeType};

/// VTK: `vtkOverrideInformationCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct OverrideInformationCollection {
    object: Object,
    current: usize,
    override_information: Vec<NonNull<OverrideInformation>>,
}

impl OverrideInformationCollection {
    /// VTK: `vtkOverrideInformationCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkOverrideInformationCollection"),
            current: 0,
            override_information: Vec::new(),
        }
    }

    /// VTK: `vtkOverrideInformationCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Number Of Items: {}\n", self.override_information.len())
    }

    /// VTK: `vtkOverrideInformationCollection::AddItem`.
    pub fn add_item(&mut self, information: *mut OverrideInformation) {
        let Some(information) = NonNull::new(information) else {
            return;
        };
        self.override_information.push(information);
        self.modified();
    }

    /// VTK: `vtkOverrideInformationCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut OverrideInformation {
        if self.current >= self.override_information.len() {
            return ptr::null_mut();
        }
        let information = self.override_information[self.current].as_ptr();
        self.current += 1;
        information
    }

    /// VTK: `vtkOverrideInformationCollection::GetNextOverrideInformation`.
    pub fn get_next_override_information(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> *mut OverrideInformation {
        if *cookie >= self.override_information.len() {
            return ptr::null_mut();
        }
        let information = self.override_information[*cookie].as_ptr();
        *cookie += 1;
        information
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
        self.override_information.len() as i32
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

impl Default for OverrideInformationCollection {
    fn default() -> Self {
        Self::new()
    }
}
