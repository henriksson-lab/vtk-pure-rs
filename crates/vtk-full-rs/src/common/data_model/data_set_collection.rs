use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};

use super::DataSet;

/// VTK: `vtkDataSetCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct DataSetCollection {
    object: Object,
    current: usize,
    data_sets: Vec<NonNull<DataSet>>,
}

impl DataSetCollection {
    /// VTK: `vtkDataSetCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataSetCollection"),
            current: 0,
            data_sets: Vec::new(),
        }
    }

    /// VTK: `vtkDataSetCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkDataSetCollection::AddItem(vtkDataSet*)`.
    pub fn add_item(&mut self, data_set: *mut DataSet) {
        let Some(data_set) = NonNull::new(data_set) else {
            return;
        };
        self.data_sets.push(data_set);
        self.modified();
    }

    /// VTK: `vtkDataSetCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut DataSet {
        self.get_next_data_set()
    }

    /// VTK: `vtkDataSetCollection::GetNextDataSet()`.
    pub fn get_next_data_set(&mut self) -> *mut DataSet {
        if self.current >= self.data_sets.len() {
            return ptr::null_mut();
        }
        let data_set = self.data_sets[self.current].as_ptr();
        self.current += 1;
        data_set
    }

    /// VTK: `vtkDataSetCollection::GetItem`.
    pub fn get_item(&self, i: i32) -> *mut DataSet {
        self.get_data_set(i)
    }

    /// VTK: `vtkDataSetCollection::GetDataSet`.
    pub fn get_data_set(&self, i: i32) -> *mut DataSet {
        if i < 0 {
            return ptr::null_mut();
        }
        self.data_sets
            .get(i as usize)
            .map_or(ptr::null_mut(), |data_set| data_set.as_ptr())
    }

    /// VTK: `vtkDataSetCollection::GetNextDataSet(vtkCollectionSimpleIterator&)`.
    pub fn get_next_data_set_with_cookie(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> *mut DataSet {
        if *cookie >= self.data_sets.len() {
            return ptr::null_mut();
        }
        let data_set = self.data_sets[*cookie].as_ptr();
        *cookie += 1;
        data_set
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.data_sets.len() as i32
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

impl Default for DataSetCollection {
    fn default() -> Self {
        Self::new()
    }
}
