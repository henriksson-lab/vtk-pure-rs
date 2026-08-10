use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};
use crate::common::data_model::StructuredPoints;

/// VTK: `vtkStructuredPointsCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredPointsCollection {
    object: Object,
    current: usize,
    structured_points: Vec<NonNull<StructuredPoints>>,
}

impl StructuredPointsCollection {
    /// VTK: `vtkStructuredPointsCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkStructuredPointsCollection"),
            current: 0,
            structured_points: Vec::new(),
        }
    }

    /// VTK: `vtkStructuredPointsCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Number Of Items: {}\n", self.structured_points.len())
    }

    /// VTK: `vtkStructuredPointsCollection::AddItem`.
    pub fn add_item(&mut self, structured_points: *mut StructuredPoints) {
        let Some(structured_points) = NonNull::new(structured_points) else {
            return;
        };
        self.structured_points.push(structured_points);
        self.modified();
    }

    /// VTK: `vtkStructuredPointsCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut StructuredPoints {
        if self.current >= self.structured_points.len() {
            return ptr::null_mut();
        }
        let structured_points = self.structured_points[self.current].as_ptr();
        self.current += 1;
        structured_points
    }

    /// VTK: `vtkStructuredPointsCollection::GetNextStructuredPoints`.
    pub fn get_next_structured_points(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> *mut StructuredPoints {
        if *cookie >= self.structured_points.len() {
            return ptr::null_mut();
        }
        let structured_points = self.structured_points[*cookie].as_ptr();
        *cookie += 1;
        structured_points
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
        self.structured_points.len() as i32
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

impl Default for StructuredPointsCollection {
    fn default() -> Self {
        Self::new()
    }
}
