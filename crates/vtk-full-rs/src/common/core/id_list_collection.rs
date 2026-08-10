use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, IdList, Object, VtkMTimeType};

/// VTK: `vtkIdListCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct IdListCollection {
    object: Object,
    current: usize,
    id_lists: Vec<NonNull<IdList>>,
}

impl IdListCollection {
    /// VTK: `vtkIdListCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkIdListCollection"),
            current: 0,
            id_lists: Vec::new(),
        }
    }

    /// VTK: `vtkIdListCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Number Of Items: {}\n", self.id_lists.len())
    }

    /// VTK: `vtkIdListCollection::AddItem`.
    pub fn add_item(&mut self, id_list: *mut IdList) {
        let Some(id_list) = NonNull::new(id_list) else {
            return;
        };
        self.id_lists.push(id_list);
        self.modified();
    }

    /// VTK: `vtkCollection::RemoveAllItems`.
    pub fn remove_all_items(&mut self) {
        self.id_lists.clear();
        self.current = 0;
        self.modified();
    }

    /// VTK: `vtkIdListCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut IdList {
        if self.current >= self.id_lists.len() {
            return ptr::null_mut();
        }
        let id_list = self.id_lists[self.current].as_ptr();
        self.current += 1;
        id_list
    }

    /// VTK: `vtkIdListCollection::GetItem`.
    pub fn get_item(&self, i: i32) -> *mut IdList {
        if i < 0 {
            return ptr::null_mut();
        }
        self.id_lists
            .get(i as usize)
            .map_or(ptr::null_mut(), |id_list| id_list.as_ptr())
    }

    /// VTK: `vtkIdListCollection::GetNextIdList`.
    pub fn get_next_id_list(&self, cookie: &mut CollectionSimpleIterator) -> *mut IdList {
        if *cookie >= self.id_lists.len() {
            return ptr::null_mut();
        }
        let id_list = self.id_lists[*cookie].as_ptr();
        *cookie += 1;
        id_list
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
        self.id_lists.len() as i32
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

impl Default for IdListCollection {
    fn default() -> Self {
        Self::new()
    }
}
