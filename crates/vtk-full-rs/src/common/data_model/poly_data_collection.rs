use std::ptr;

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};

use super::PolyDataHandle;

/// VTK: `vtkPolyDataCollection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyDataCollection {
    object: Object,
    current: usize,
    poly_data: Vec<PolyDataHandle>,
}

impl PolyDataCollection {
    /// VTK: `vtkPolyDataCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPolyDataCollection"),
            current: 0,
            poly_data: Vec::new(),
        }
    }

    /// VTK: `vtkPolyDataCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkPolyDataCollection::AddItem(vtkPolyData*)`.
    pub fn add_item(&mut self, poly_data: PolyDataHandle) {
        if poly_data.is_null() {
            return;
        }
        self.poly_data.push(poly_data);
        self.modified();
    }

    /// VTK: `vtkPolyDataCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> PolyDataHandle {
        if self.current >= self.poly_data.len() {
            return ptr::null_mut();
        }
        let poly_data = self.poly_data[self.current];
        self.current += 1;
        poly_data
    }

    /// VTK: `vtkPolyDataCollection::GetNextPolyData(vtkCollectionSimpleIterator&)`.
    pub fn get_next_poly_data(&self, cookie: &mut CollectionSimpleIterator) -> PolyDataHandle {
        if *cookie >= self.poly_data.len() {
            return ptr::null_mut();
        }
        let poly_data = self.poly_data[*cookie];
        *cookie += 1;
        poly_data
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.poly_data.len() as i32
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

impl Default for PolyDataCollection {
    fn default() -> Self {
        Self::new()
    }
}
