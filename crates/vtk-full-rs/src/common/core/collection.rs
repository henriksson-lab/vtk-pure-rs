use std::{
    cmp::Ordering,
    ptr::{self, NonNull},
};

use super::{
    collection_iterator::CollectionIterator,
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkCollectionSimpleIterator`.
pub type CollectionSimpleIterator = usize;

/// VTK: `vtkCollection`.
#[derive(Debug)]
pub struct Collection {
    object: Object,
    current: usize,
    objects: Vec<NonNull<Object>>,
}

impl Collection {
    /// VTK: `vtkCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCollection"),
            current: 0,
            objects: Vec::new(),
        }
    }

    /// VTK: `vtkCollection::AddItem`.
    ///
    /// # Safety
    ///
    /// `item` must point to a live `Object` that remains valid while it is in
    /// the collection. This mirrors VTK's raw `vtkObject*` storage contract.
    pub unsafe fn add_item(&mut self, item: *mut Object) {
        let item = NonNull::new(item).expect("vtkCollection::AddItem item must not be null");
        self.objects.push(item);
        unsafe {
            item.as_ptr().as_mut().unwrap().register();
        }
        self.modified();
    }

    /// VTK: `vtkCollection::InsertItem`.
    ///
    /// # Safety
    ///
    /// `item` must point to a live `Object` that remains valid while it is in
    /// the collection. This mirrors VTK's raw `vtkObject*` storage contract.
    pub unsafe fn insert_item(&mut self, i: i32, item: *mut Object) {
        if self.objects.is_empty() {
            return;
        }

        let item = NonNull::new(item).expect("vtkCollection::InsertItem item must not be null");
        let Some(index) = self.insert_index(i) else {
            return;
        };

        self.objects.insert(index, item);
        unsafe {
            item.as_ptr().as_mut().unwrap().register();
        }
        self.modified();
    }

    /// VTK: `vtkCollection::ReplaceItem`.
    ///
    /// # Safety
    ///
    /// `item` must point to a live `Object` that remains valid while it is in
    /// the collection. Any existing stored object pointers must still be valid
    /// so their VTK reference counts can be decremented.
    pub unsafe fn replace_item(&mut self, i: i32, item: *mut Object) {
        if i < 0 {
            return;
        }
        let index = i as usize;
        if index >= self.objects.len() {
            return;
        }

        let item = NonNull::new(item).expect("vtkCollection::ReplaceItem item must not be null");
        unsafe {
            self.objects[index].as_ptr().as_mut().unwrap().unregister();
            item.as_ptr().as_mut().unwrap().register();
        }
        self.objects[index] = item;
        self.modified();
    }

    /// VTK: `vtkCollection::RemoveItem(int)`.
    ///
    /// # Safety
    ///
    /// Stored object pointers must still be valid so their VTK reference counts
    /// can be decremented.
    pub unsafe fn remove_item_at(&mut self, i: i32) {
        if i < 0 {
            return;
        }
        let index = i as usize;
        if index >= self.objects.len() {
            return;
        }
        unsafe {
            self.remove_item_at_index(index);
        }
    }

    /// VTK: `vtkCollection::RemoveItem(vtkObject*)`.
    ///
    /// # Safety
    ///
    /// Stored object pointers must still be valid so their VTK reference counts
    /// can be decremented.
    pub unsafe fn remove_item(&mut self, item: *mut Object) {
        let Some(item) = NonNull::new(item) else {
            return;
        };
        if let Some(index) = self.objects.iter().position(|object| *object == item) {
            unsafe {
                self.remove_item_at_index(index);
            }
        }
    }

    /// VTK: `vtkCollection::RemoveAllItems`.
    ///
    /// # Safety
    ///
    /// Stored object pointers must still be valid so their VTK reference counts
    /// can be decremented.
    pub unsafe fn remove_all_items(&mut self) {
        if self.objects.is_empty() {
            return;
        }
        for object in &mut self.objects {
            unsafe {
                object.as_ptr().as_mut().unwrap().unregister();
            }
        }
        self.objects.clear();
        self.current = 0;
        self.modified();
    }

    /// VTK: `vtkCollection::IsItemPresent`.
    pub fn is_item_present(&self, item: *mut Object) -> i32 {
        self.index_of_first_occurrence(item) + 1
    }

    /// VTK: `vtkCollection::IndexOfFirstOccurence`.
    pub fn index_of_first_occurence(&self, item: *mut Object) -> i32 {
        self.index_of_first_occurrence(item)
    }

    /// VTK: `vtkCollection::IndexOfFirstOccurrence`.
    pub fn index_of_first_occurrence(&self, item: *mut Object) -> i32 {
        let Some(item) = NonNull::new(item) else {
            return -1;
        };
        self.objects
            .iter()
            .position(|object| *object == item)
            .map_or(-1, |index| index as i32)
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.objects.len() as i32
    }

    /// VTK: `vtkCollection::GetItemAsObject`.
    pub fn get_item_as_object(&self, i: i32) -> *mut Object {
        if i < 0 {
            return ptr::null_mut();
        }
        self.objects
            .get(i as usize)
            .map_or(ptr::null_mut(), |object| object.as_ptr())
    }

    /// VTK: `vtkCollection::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.current = 0;
    }

    /// VTK: `vtkCollection::InitTraversal(vtkCollectionSimpleIterator&)`.
    pub fn init_traversal_cookie(&self, cookie: &mut CollectionSimpleIterator) {
        *cookie = 0;
    }

    /// VTK: `vtkCollection::GetNextItemAsObject`.
    pub fn get_next_item_as_object(&mut self) -> *mut Object {
        if self.current >= self.objects.len() {
            return ptr::null_mut();
        }
        let object = self.objects[self.current].as_ptr();
        self.current += 1;
        object
    }

    /// VTK: `vtkCollection::GetNextItemAsObject(vtkCollectionSimpleIterator&)`.
    pub fn get_next_item_as_object_cookie(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> *mut Object {
        if *cookie >= self.objects.len() {
            return ptr::null_mut();
        }
        let object = self.objects[*cookie].as_ptr();
        *cookie += 1;
        object
    }

    /// VTK: `vtkCollection::NewIterator`.
    ///
    /// # Safety
    ///
    /// The returned iterator stores a raw pointer to this collection. The
    /// collection must outlive the iterator, matching VTK's object-pointer
    /// ownership contract.
    pub unsafe fn new_iterator(&mut self) -> CollectionIterator {
        let mut iterator = CollectionIterator::new();
        unsafe {
            iterator.set_collection(self as *mut Collection);
        }
        iterator
    }

    /// VTK: `vtkCollection::Sort`.
    pub fn sort<F>(&mut self, mut f: F)
    where
        F: FnMut(*mut Object, *mut Object) -> bool,
    {
        self.objects.sort_by(|left, right| {
            if f(left.as_ptr(), right.as_ptr()) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
    }

    /// VTK: `vtkCollection::begin`/`vtkCollection::end`.
    pub fn iter(&self) -> impl Iterator<Item = *mut Object> + '_ {
        self.objects.iter().map(|object| object.as_ptr())
    }

    /// VTK: `vtkCollection::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        true
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCollection::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCollection" || Object::is_type_of(name)
    }

    /// VTK: `vtkCollection::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCollection::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkCollection" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCollection::GetNumberOfGenerationsFromBase`.
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

    fn insert_index(&self, i: i32) -> Option<usize> {
        if i < 0 {
            Some(0)
        } else if i as usize >= self.objects.len() {
            None
        } else {
            Some(i as usize + 1)
        }
    }

    unsafe fn remove_item_at_index(&mut self, index: usize) {
        if index < self.current {
            self.current -= 1;
        }
        let object = self.objects.remove(index);
        unsafe {
            object.as_ptr().as_mut().unwrap().unregister();
        }
        self.modified();
    }

    pub(crate) fn len(&self) -> usize {
        self.objects.len()
    }

    pub(crate) fn object_at_raw(&self, index: usize) -> *mut Object {
        self.objects
            .get(index)
            .map_or(ptr::null_mut(), |object| object.as_ptr())
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Collection {
    fn drop(&mut self) {
        unsafe {
            self.remove_all_items();
        }
    }
}
