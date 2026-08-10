use std::{ptr, ptr::NonNull};

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};

use super::Plane;

/// VTK: `vtkPlaneCollection`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaneCollection {
    object: Object,
    current: usize,
    planes: Vec<NonNull<Plane>>,
}

impl PlaneCollection {
    /// VTK: `vtkPlaneCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPlaneCollection"),
            current: 0,
            planes: Vec::new(),
        }
    }

    /// VTK: `vtkPlaneCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
    }

    /// VTK: `vtkPlaneCollection::AddItem(vtkPlane*)`.
    pub fn add_item(&mut self, plane: *mut Plane) {
        let Some(plane) = NonNull::new(plane) else {
            return;
        };
        self.planes.push(plane);
        self.modified();
    }

    /// VTK: `vtkPlaneCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> *mut Plane {
        if self.current >= self.planes.len() {
            return ptr::null_mut();
        }
        let plane = self.planes[self.current].as_ptr();
        self.current += 1;
        plane
    }

    /// VTK: `vtkPlaneCollection::GetItem`.
    pub fn get_item(&self, i: i32) -> *mut Plane {
        if i < 0 {
            return ptr::null_mut();
        }
        self.planes
            .get(i as usize)
            .map_or(ptr::null_mut(), |plane| plane.as_ptr())
    }

    /// VTK: `vtkPlaneCollection::GetNextPlane(vtkCollectionSimpleIterator&)`.
    pub fn get_next_plane(&self, cookie: &mut CollectionSimpleIterator) -> *mut Plane {
        if *cookie >= self.planes.len() {
            return ptr::null_mut();
        }
        let plane = self.planes[*cookie].as_ptr();
        *cookie += 1;
        plane
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.planes.len() as i32
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

impl Default for PlaneCollection {
    fn default() -> Self {
        Self::new()
    }
}
