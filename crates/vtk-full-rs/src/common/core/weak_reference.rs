use crate::common::core::{Object, VtkMTimeType, WeakPointerBase};

/// VTK: `vtkWeakReference`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeakReference {
    object: Object,
    referenced_object: WeakPointerBase,
    referenced_typed_object: *mut Object,
}

impl WeakReference {
    /// VTK: `vtkWeakReference::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkWeakReference"),
            referenced_object: WeakPointerBase::new(),
            referenced_typed_object: std::ptr::null_mut(),
        }
    }

    /// VTK: `vtkWeakReference::Set`.
    pub fn set(&mut self, object: *mut Object) {
        self.referenced_typed_object = object;
        let object_base = if object.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { object.as_mut().unwrap().object_base_mut_ptr() }
        };
        self.referenced_object.assign_object_base(object_base);
    }

    /// VTK: `vtkWeakReference::Get`.
    pub fn get(&self) -> *mut Object {
        if self.referenced_object.get_pointer().is_null() {
            std::ptr::null_mut()
        } else {
            self.referenced_typed_object
        }
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

impl Default for WeakReference {
    fn default() -> Self {
        Self::new()
    }
}
