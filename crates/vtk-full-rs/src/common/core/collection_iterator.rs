use std::ptr::{self, NonNull};

use super::{
    collection::Collection,
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkCollectionIterator`.
#[derive(Debug)]
pub struct CollectionIterator {
    object: Object,
    collection: Option<NonNull<Collection>>,
    iterator: usize,
}

impl CollectionIterator {
    /// VTK: `vtkCollectionIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCollectionIterator"),
            collection: None,
            iterator: 0,
        }
    }

    /// VTK: `vtkCollectionIterator::SetCollection`.
    ///
    /// # Safety
    ///
    /// `collection` must either be null or point to a live `Collection` that
    /// remains valid while this iterator references it.
    pub unsafe fn set_collection(&mut self, collection: *mut Collection) {
        let next = NonNull::new(collection);
        if self.collection == next {
            self.go_to_first_item();
            return;
        }

        let old = self.collection;
        self.collection = next;
        if let Some(mut new_collection) = self.collection {
            unsafe {
                new_collection.as_mut().register();
            }
        }
        if let Some(mut old) = old {
            unsafe {
                old.as_mut().unregister();
            }
        }
        self.modified();
        self.go_to_first_item();
    }

    /// VTK: `vtkCollectionIterator::GetCollection`.
    pub fn get_collection(&self) -> *mut Collection {
        self.collection
            .map_or(ptr::null_mut(), |collection| collection.as_ptr())
    }

    /// VTK: `vtkCollectionIterator::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.go_to_first_item();
    }

    /// VTK: `vtkCollectionIterator::GoToFirstItem`.
    pub fn go_to_first_item(&mut self) {
        if self.collection.is_some() {
            self.iterator = 0;
        }
    }

    /// VTK: `vtkCollectionIterator::GoToNextItem`.
    pub fn go_to_next_item(&mut self) {
        if let Some(collection) = self.collection {
            let collection = unsafe { collection.as_ref() };
            if self.iterator < collection.len() {
                self.iterator += 1;
            }
        }
    }

    /// VTK: `vtkCollectionIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> i32 {
        let Some(collection) = self.collection else {
            return 1;
        };
        let collection = unsafe { collection.as_ref() };
        i32::from(self.iterator >= collection.len())
    }

    /// VTK: `vtkCollectionIterator::GetCurrentObject`.
    pub fn get_current_object(&self) -> *mut Object {
        let Some(collection) = self.collection else {
            return ptr::null_mut();
        };
        let collection = unsafe { collection.as_ref() };
        if self.iterator < collection.len() {
            collection.object_at_raw(self.iterator)
        } else {
            ptr::null_mut()
        }
    }

    /// VTK: `vtkCollectionIterator::GetObjectInternal`.
    #[allow(dead_code)]
    pub(crate) fn get_object_internal(&self) -> *mut Object {
        self.get_current_object()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCollectionIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCollectionIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkCollectionIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCollectionIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkCollectionIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCollectionIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        Object::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Object::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Object::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        Object::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.object.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.object.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.object.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.object.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        Object::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.object.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.object.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.object.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.object.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for CollectionIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CollectionIterator {
    fn drop(&mut self) {
        unsafe {
            self.set_collection(ptr::null_mut());
        }
    }
}
