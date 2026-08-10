use std::{cell::RefCell, ptr, rc::Rc};

use super::{
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

const VTK_TMP_ARRAY_SIZE: usize = 500;

/// VTK: `vtkIdList`.
#[derive(Debug, Clone)]
pub struct IdList {
    object: Object,
    number_of_ids: usize,
    buffer: Rc<RefCell<Vec<VtkIdType>>>,
}

impl IdList {
    /// VTK: `vtkIdList::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkIdList"),
            number_of_ids: 0,
            buffer: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// VTK: `vtkIdList::Release`.
    pub fn release(&mut self) -> Vec<VtkIdType> {
        let released = self.buffer.borrow().clone();
        self.initialize();
        released
    }

    /// VTK: `vtkIdList::InitializeMemory`.
    #[allow(dead_code)]
    pub(crate) fn initialize_memory(&mut self) {
        self.buffer = Rc::new(RefCell::new(Vec::new()));
    }

    /// VTK: `vtkIdList::Initialize`.
    pub fn initialize(&mut self) {
        self.reset();
        self.squeeze();
    }

    /// VTK: `vtkIdList::AllocateInternal`.
    #[allow(dead_code)]
    pub(crate) fn allocate_internal(&mut self, size: VtkIdType, number_of_ids: VtkIdType) -> bool {
        self.initialize();
        if self.reserve(size) {
            self.number_of_ids = id_count_to_usize(number_of_ids);
            true
        } else {
            false
        }
    }

    /// VTK: `vtkIdList::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, _strategy: i32) -> bool {
        self.number_of_ids = 0;
        if size > self.get_capacity() || size == 0 {
            let size = id_count_to_usize(size);
            *self.buffer.borrow_mut() = vec![0; size];
        }
        true
    }

    /// VTK: `vtkIdList::Reserve`.
    pub fn reserve(&mut self, size: VtkIdType) -> bool {
        if size <= self.get_capacity() {
            return true;
        }
        let size = id_count_to_usize(size);
        let new_size = self.buffer.borrow().len() + size;
        self.buffer.borrow_mut().resize(new_size, 0);
        true
    }

    /// VTK: `vtkIdList::GetNumberOfIds`.
    pub fn get_number_of_ids(&self) -> VtkIdType {
        self.number_of_ids as VtkIdType
    }

    /// VTK: `vtkIdList::GetId`.
    pub fn get_id(&self, i: VtkIdType) -> VtkIdType {
        let i = vtk_id_to_usize(i);
        assert!(i < self.number_of_ids, "id index out of range");
        self.buffer.borrow()[i]
    }

    /// VTK: `vtkIdList::FindIdLocation`.
    pub fn find_id_location(&self, id: VtkIdType) -> VtkIdType {
        self.buffer.borrow()[..self.number_of_ids]
            .iter()
            .position(|&value| value == id)
            .map_or(-1, |index| index as VtkIdType)
    }

    /// VTK: `vtkIdList::SetNumberOfIds`.
    pub fn set_number_of_ids(&mut self, number: VtkIdType) {
        if self.reserve(number) {
            self.number_of_ids = id_count_to_usize(number);
        }
    }

    /// VTK: `vtkIdList::SetId`.
    pub fn set_id(&mut self, i: VtkIdType, id: VtkIdType) {
        let i = vtk_id_to_usize(i);
        assert!(i < self.number_of_ids, "id index out of range");
        self.buffer.borrow_mut()[i] = id;
    }

    /// VTK: `vtkIdList::InsertId`.
    pub fn insert_id(&mut self, i: VtkIdType, id: VtkIdType) {
        let i = vtk_id_to_usize(i);
        if i >= self.buffer.borrow().len() {
            self.reserve((i + 1) as VtkIdType);
        }
        self.buffer.borrow_mut()[i] = id;
        if i >= self.number_of_ids {
            self.number_of_ids = i + 1;
        }
    }

    /// VTK: `vtkIdList::InsertNextId`.
    pub fn insert_next_id(&mut self, id: VtkIdType) -> VtkIdType {
        if self.number_of_ids >= self.buffer.borrow().len()
            && !self.reserve((self.number_of_ids + 1) as VtkIdType)
        {
            return self.number_of_ids.saturating_sub(1) as VtkIdType;
        }
        self.buffer.borrow_mut()[self.number_of_ids] = id;
        self.number_of_ids += 1;
        (self.number_of_ids - 1) as VtkIdType
    }

    /// VTK: `vtkIdList::InsertUniqueId`.
    pub fn insert_unique_id(&mut self, id: VtkIdType) -> VtkIdType {
        let location = self.find_id_location(id);
        if location != -1 {
            location
        } else {
            self.insert_next_id(id)
        }
    }

    /// VTK: `vtkIdList::Sort`.
    pub fn sort(&mut self) {
        if self.number_of_ids < 2 {
            return;
        }
        self.buffer.borrow_mut()[..self.number_of_ids].sort();
    }

    /// VTK: `vtkIdList::Fill`.
    pub fn fill(&mut self, value: VtkIdType) {
        if self.number_of_ids < 1 {
            return;
        }
        self.buffer.borrow_mut()[..self.number_of_ids].fill(value);
    }

    /// VTK: `vtkIdList::GetPointer`.
    pub fn get_pointer(&mut self, i: VtkIdType) -> *mut VtkIdType {
        let i = vtk_id_to_usize(i);
        self.buffer.borrow_mut().as_mut_ptr().wrapping_add(i)
    }

    /// VTK: `vtkIdList::WritePointer`.
    pub fn write_pointer(&mut self, i: VtkIdType, number: VtkIdType) -> *mut VtkIdType {
        let i = vtk_id_to_usize(i);
        let number = id_count_to_usize(number);
        let new_size = i + number;
        if new_size > self.buffer.borrow().len() {
            self.reserve(new_size as VtkIdType);
        }
        self.number_of_ids = self.number_of_ids.max(new_size);
        self.buffer.borrow_mut().as_mut_ptr().wrapping_add(i)
    }

    /// VTK: `vtkIdList::SetList`.
    pub fn set_list(&mut self, values: Vec<VtkIdType>, _save: bool, _delete_method: i32) {
        self.number_of_ids = values.len();
        self.buffer = Rc::new(RefCell::new(values));
    }

    /// VTK: `vtkIdList::SetArray`.
    pub fn set_array(&mut self, values: Vec<VtkIdType>, manage_memory: bool) {
        self.set_list(values, !manage_memory, DeleteMethod::DataArrayDelete as i32);
    }

    /// VTK: `vtkIdList::Reset`.
    pub fn reset(&mut self) {
        self.number_of_ids = 0;
    }

    /// VTK: `vtkIdList::Squeeze`.
    pub fn squeeze(&mut self) {
        if self.buffer.borrow().len() > self.number_of_ids {
            self.buffer.borrow_mut().truncate(self.number_of_ids);
        }
    }

    /// VTK: `vtkIdList::ShallowCopy`.
    pub fn shallow_copy(&mut self, list: &Self) {
        self.number_of_ids = list.number_of_ids;
        if !Rc::ptr_eq(&self.buffer, &list.buffer) {
            self.buffer = Rc::clone(&list.buffer);
        }
    }

    /// VTK: `vtkIdList::DeepCopy`.
    pub fn deep_copy(&mut self, ids: &Self) {
        self.set_number_of_ids(ids.get_number_of_ids());
        if ids.number_of_ids > 0 {
            self.buffer.borrow_mut()[..ids.number_of_ids]
                .copy_from_slice(&ids.buffer.borrow()[..ids.number_of_ids]);
        }
        self.squeeze();
    }

    /// VTK: `vtkIdList::DeleteId`.
    pub fn delete_id(&mut self, id: VtkIdType) {
        let mut i = 0;
        while i < self.number_of_ids {
            while i < self.number_of_ids && self.buffer.borrow()[i] != id {
                i += 1;
            }
            if i < self.number_of_ids {
                let last = self.buffer.borrow()[self.number_of_ids - 1];
                self.buffer.borrow_mut()[i] = last;
                self.number_of_ids -= 1;
            }
        }
    }

    /// VTK: `vtkIdList::IsId`.
    pub fn is_id(&self, id: VtkIdType) -> VtkIdType {
        self.find_id_location(id)
    }

    /// VTK: `vtkIdList::IntersectWith`.
    pub fn intersect_with(&mut self, other_ids: &Self) {
        let this_num_ids = self.number_of_ids;
        let this_ids = if this_num_ids <= VTK_TMP_ARRAY_SIZE {
            self.buffer.borrow()[..this_num_ids].to_vec()
        } else {
            self.iter().collect()
        };
        self.reset();
        for id in this_ids {
            if other_ids.is_id(id) != -1 {
                self.insert_next_id(id);
            }
        }
    }

    /// VTK: `vtkIdList::Resize`.
    pub fn resize(&mut self, size: VtkIdType) -> *mut VtkIdType {
        if size <= 0 {
            self.initialize();
            return ptr::null_mut();
        }
        let size = id_count_to_usize(size);
        if self.buffer.borrow().len() >= size {
            self.number_of_ids = size;
            self.squeeze();
            self.buffer.borrow_mut().as_mut_ptr()
        } else {
            self.reserve(size as VtkIdType);
            self.buffer.borrow_mut().as_mut_ptr()
        }
    }

    /// VTK: `vtkIdList::GetCapacity`.
    pub fn get_capacity(&self) -> VtkIdType {
        self.buffer.borrow().len() as VtkIdType
    }

    /// VTK: `vtkIdList::begin`/`end`.
    pub fn iter(&self) -> impl Iterator<Item = VtkIdType> + '_ {
        let values = self.buffer.borrow()[..self.number_of_ids].to_vec();
        values.into_iter()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkIdList::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkIdList" || Object::is_type_of(name)
    }

    /// VTK: `vtkIdList::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkIdList::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkIdList" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name) as VtkIdType,
        }
    }

    /// VTK: `vtkIdList::GetNumberOfGenerationsFromBase`.
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

impl Default for IdList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteMethod {
    DataArrayFree = 0,
    DataArrayDelete = 1,
    DataArrayAlignedFree = 2,
    DataArrayUserDefined = 3,
}

fn id_count_to_usize(count: VtkIdType) -> usize {
    usize::try_from(count.max(0)).expect("vtkIdType count must fit usize")
}

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}
