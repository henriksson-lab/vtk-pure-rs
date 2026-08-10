use std::{fmt, ptr};

use crate::common::core::ObjectBase;

/// VTK: `vtkSmartPointerBase`.
#[derive(Debug)]
pub struct SmartPointerBase {
    object: *mut ObjectBase,
}

impl SmartPointerBase {
    /// VTK: `vtkSmartPointerBase::vtkSmartPointerBase()`.
    pub fn new() -> Self {
        Self {
            object: ptr::null_mut(),
        }
    }

    /// VTK: `vtkSmartPointerBase::vtkSmartPointerBase(vtkObjectBase*)`.
    pub fn from_object_base(object: *mut ObjectBase) -> Self {
        let smart_pointer = Self { object };
        smart_pointer.register_object();
        smart_pointer
    }

    /// VTK: `vtkSmartPointerBase::GetPointer`.
    pub fn get_pointer(&self) -> *mut ObjectBase {
        self.object
    }

    /// VTK: `vtkSmartPointerBase::operator=(vtkObjectBase*)`.
    pub fn assign_object_base(&mut self, object: *mut ObjectBase) -> &mut Self {
        if self.object != object {
            Self::from_object_base(object).swap(self);
        }
        self
    }

    /// VTK: `vtkSmartPointerBase::operator=(const vtkSmartPointerBase&)`.
    pub fn assign(&mut self, other: &SmartPointerBase) -> &mut Self {
        if !ptr::eq(self, other) && self.object != other.object {
            other.clone().swap(self);
        }
        self
    }

    /// VTK: `vtkSmartPointerBase::vtkSmartPointerBase(vtkObjectBase*, const NoReference&)`.
    #[allow(dead_code)]
    pub(crate) fn from_object_base_no_reference(object: *mut ObjectBase) -> Self {
        Self { object }
    }

    /// VTK: `vtkSmartPointerBase::Swap`.
    fn swap(&mut self, other: &mut SmartPointerBase) {
        std::mem::swap(&mut self.object, &mut other.object);
    }

    /// VTK: `vtkSmartPointerBase::Register`.
    fn register_object(&self) {
        if !self.object.is_null() {
            unsafe {
                self.object.as_mut().unwrap().register();
            }
        }
    }
}

impl Clone for SmartPointerBase {
    fn clone(&self) -> Self {
        Self::from_object_base(self.object)
    }
}

impl Default for SmartPointerBase {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for SmartPointerBase {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl Eq for SmartPointerBase {}

impl PartialOrd for SmartPointerBase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.object as usize).partial_cmp(&(other.object as usize))
    }
}

impl fmt::Pointer for SmartPointerBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.object as *const ObjectBase), f)
    }
}

impl Drop for SmartPointerBase {
    fn drop(&mut self) {
        let object = self.object;
        if !object.is_null() {
            self.object = ptr::null_mut();
            unsafe {
                object.as_mut().unwrap().unregister();
            }
        }
    }
}
