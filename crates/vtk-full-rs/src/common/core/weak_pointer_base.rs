use std::{
    fmt, ptr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};

use crate::common::core::{ObjectBase, WeakPointerSlot};

/// VTK: `vtkWeakPointerBase`.
#[derive(Debug)]
pub struct WeakPointerBase {
    object: WeakPointerSlot,
}

impl WeakPointerBase {
    /// VTK: `vtkWeakPointerBase::vtkWeakPointerBase()`.
    pub fn new() -> Self {
        Self {
            object: Arc::new(AtomicPtr::new(ptr::null_mut())),
        }
    }

    /// VTK: `vtkWeakPointerBase::vtkWeakPointerBase(vtkObjectBase*)`.
    pub fn from_object_base(object: *mut ObjectBase) -> Self {
        let weak_pointer = Self {
            object: Arc::new(AtomicPtr::new(object)),
        };
        weak_pointer.add_to_object();
        weak_pointer
    }

    /// VTK: `vtkWeakPointerBase::GetPointer`.
    pub fn get_pointer(&self) -> *mut ObjectBase {
        self.object.load(Ordering::Relaxed)
    }

    /// VTK: `vtkWeakPointerBase::operator=(vtkObjectBase*)`.
    pub fn assign_object_base(&mut self, object: *mut ObjectBase) -> &mut Self {
        if self.get_pointer() != object {
            self.remove_from_object();
            self.object.store(object, Ordering::Relaxed);
            self.add_to_object();
        }
        self
    }

    /// VTK: `vtkWeakPointerBase::operator=(const vtkWeakPointerBase&)`.
    pub fn assign(&mut self, other: &WeakPointerBase) -> &mut Self {
        if !ptr::eq(self, other) {
            self.assign_object_base(other.get_pointer());
        }
        self
    }

    fn add_to_object(&self) {
        let object = self.get_pointer();
        if !object.is_null() {
            unsafe {
                object.as_ref().unwrap().add_weak_pointer(&self.object);
            }
        }
    }

    fn remove_from_object(&self) {
        let object = self.get_pointer();
        if !object.is_null() {
            unsafe {
                object.as_ref().unwrap().remove_weak_pointer(&self.object);
            }
        }
    }
}

impl Clone for WeakPointerBase {
    fn clone(&self) -> Self {
        Self::from_object_base(self.get_pointer())
    }
}

impl Default for WeakPointerBase {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for WeakPointerBase {
    fn eq(&self, other: &Self) -> bool {
        self.get_pointer() == other.get_pointer()
    }
}

impl Eq for WeakPointerBase {}

impl PartialOrd for WeakPointerBase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.get_pointer() as usize).partial_cmp(&(other.get_pointer() as usize))
    }
}

impl fmt::Pointer for WeakPointerBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.get_pointer() as *const ObjectBase), f)
    }
}

impl Drop for WeakPointerBase {
    fn drop(&mut self) {
        self.remove_from_object();
        self.object.store(ptr::null_mut(), Ordering::Relaxed);
    }
}
