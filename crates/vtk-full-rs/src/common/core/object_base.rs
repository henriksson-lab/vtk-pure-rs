use std::{
    ptr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc, Mutex, Weak,
    },
};

use super::vtk_type::VtkIdType;

const VTK_ID_MIN: VtkIdType = VtkIdType::MIN;

pub(crate) type WeakPointerSlot = Arc<AtomicPtr<ObjectBase>>;

/// VTK: `vtkObjectBase`.
#[derive(Debug)]
pub struct ObjectBase {
    class_name: &'static str,
    reference_count: i32,
    is_in_memkind: bool,
    weak_pointers: Mutex<Vec<Weak<AtomicPtr<ObjectBase>>>>,
}

impl ObjectBase {
    /// VTK: `vtkObjectBase::New`.
    pub fn new() -> Self {
        let mut object = Self {
            class_name: "vtkObjectBase",
            reference_count: 1,
            is_in_memkind: false,
            weak_pointers: Mutex::new(Vec::new()),
        };
        object.initialize_object_base();
        object
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        let mut object = Self {
            class_name,
            reference_count: 1,
            is_in_memkind: false,
            weak_pointers: Mutex::new(Vec::new()),
        };
        object.initialize_object_base();
        object
    }

    /// VTK: `vtkObjectBase::InitializeObjectBase`.
    pub fn initialize_object_base(&mut self) {}

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.class_name
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        format!("{} ({:p})", self.get_class_name(), self)
    }

    /// VTK: `vtkObjectBase::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkObjectBase"
    }

    /// VTK: `vtkObjectBase::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        if name == "vtkObjectBase" {
            0
        } else {
            VTK_ID_MIN
        }
    }

    /// VTK: `vtkObjectBase::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.register_internal(false);
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.unregister_internal(false)
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.unregister()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.unregister_internal(false)
    }

    /// VTK: `vtkObjectBase::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        false
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.reference_count
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.reference_count = reference_count;
    }

    /// VTK: `vtkObjectBase::GetUsingMemkind`.
    pub fn get_using_memkind() -> bool {
        false
    }

    /// VTK: `vtkObjectBase::GetIsInMemkind`.
    pub fn get_is_in_memkind(&self) -> bool {
        self.is_in_memkind
    }

    pub(crate) fn register_internal(&mut self, _check: bool) {
        self.reference_count = self.reference_count.saturating_add(1);
    }

    pub(crate) fn unregister_internal(&mut self, _check: bool) -> bool {
        self.reference_count = self.reference_count.saturating_sub(1);
        if self.reference_count == 0 {
            self.object_finalize();
            true
        } else {
            false
        }
    }

    pub(crate) fn object_finalize(&mut self) {
        self.clear_weak_pointers();
    }

    pub(crate) fn add_weak_pointer(&self, slot: &WeakPointerSlot) {
        let mut weak_pointers = self.weak_pointers.lock().unwrap();
        let mut already_present = false;
        weak_pointers.retain(|weak_slot| {
            let Some(existing) = weak_slot.upgrade() else {
                return false;
            };
            if Arc::ptr_eq(&existing, slot) {
                already_present = true;
            }
            true
        });
        if !already_present {
            weak_pointers.push(Arc::downgrade(slot));
        }
    }

    pub(crate) fn remove_weak_pointer(&self, slot: &WeakPointerSlot) {
        self.weak_pointers.lock().unwrap().retain(|weak_slot| {
            weak_slot
                .upgrade()
                .is_some_and(|existing| !Arc::ptr_eq(&existing, slot))
        });
    }

    pub(crate) fn clear_weak_pointers(&self) {
        for weak_slot in self.weak_pointers.lock().unwrap().drain(..) {
            if let Some(slot) = weak_slot.upgrade() {
                slot.store(ptr::null_mut(), Ordering::Relaxed);
            }
        }
    }
}

impl Default for ObjectBase {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ObjectBase {
    fn clone(&self) -> Self {
        Self {
            class_name: self.class_name,
            reference_count: self.reference_count,
            is_in_memkind: self.is_in_memkind,
            weak_pointers: Mutex::new(Vec::new()),
        }
    }
}

impl PartialEq for ObjectBase {
    fn eq(&self, other: &Self) -> bool {
        self.class_name == other.class_name
            && self.reference_count == other.reference_count
            && self.is_in_memkind == other.is_in_memkind
    }
}

impl Eq for ObjectBase {}

impl Drop for ObjectBase {
    fn drop(&mut self) {
        self.clear_weak_pointers();
    }
}
