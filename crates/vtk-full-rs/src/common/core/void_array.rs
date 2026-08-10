use std::{ffi::c_void, ptr};

use super::{
    object::Object,
    vtk_type::{VtkDataType, VtkIdType, VtkMTimeType},
};

type VoidPtr = *mut c_void;

/// VTK: `vtkVoidArray`.
#[derive(Debug, Clone, PartialEq)]
pub struct VoidArray {
    object: Object,
    number_of_pointers: VtkIdType,
    size: VtkIdType,
    array: Vec<VoidPtr>,
}

impl VoidArray {
    /// VTK: `vtkVoidArray::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkVoidArray"),
            number_of_pointers: 0,
            size: 0,
            array: Vec::new(),
        }
    }

    /// VTK: `vtkVoidArray::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    /// VTK: `vtkVoidArray::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, _extend: VtkIdType) -> bool {
        if size > self.size || !self.array.is_empty() {
            self.array.clear();
            self.size = if size > 0 { size } else { 1 };
            self.array
                .resize(vtk_id_to_usize(self.size), ptr::null_mut());
        }
        self.number_of_pointers = 0;
        true
    }

    /// VTK: `vtkVoidArray::Initialize`.
    pub fn initialize(&mut self) {
        self.array.clear();
        self.array.shrink_to_fit();
        self.size = 0;
        self.number_of_pointers = 0;
    }

    /// VTK: `vtkVoidArray::GetDataType`.
    pub fn get_data_type(&self) -> i32 {
        VtkDataType::VTK_VOID
    }

    /// VTK: `vtkVoidArray::GetDataTypeSize`.
    pub fn get_data_type_size(&self) -> usize {
        std::mem::size_of::<VoidPtr>()
    }

    /// VTK: `vtkVoidArray::SetNumberOfPointers`.
    pub fn set_number_of_pointers(&mut self, number: VtkIdType) {
        self.allocate(number, 1000);
        self.number_of_pointers = number;
    }

    /// VTK: `vtkVoidArray::GetNumberOfPointers`.
    pub fn get_number_of_pointers(&self) -> VtkIdType {
        self.number_of_pointers
    }

    /// VTK: `vtkVoidArray::GetVoidPointer`.
    pub fn get_void_pointer(&self, id: VtkIdType) -> VoidPtr {
        self.array[vtk_id_to_usize(id)]
    }

    /// VTK: `vtkVoidArray::SetVoidPointer`.
    pub fn set_void_pointer(&mut self, id: VtkIdType, ptr: VoidPtr) {
        let id = vtk_id_to_usize(id);
        self.array[id] = ptr;
    }

    /// VTK: `vtkVoidArray::InsertVoidPointer`.
    pub fn insert_void_pointer(&mut self, id: VtkIdType, ptr: VoidPtr) {
        if id < 0 {
            return;
        }
        if id >= self.size && self.resize_and_extend(id + 1).is_null() {
            return;
        }
        let id_idx = vtk_id_to_usize(id);
        self.array[id_idx] = ptr;
        if id >= self.number_of_pointers {
            self.number_of_pointers = id + 1;
        }
    }

    /// VTK: `vtkVoidArray::InsertNextVoidPointer`.
    pub fn insert_next_void_pointer(&mut self, ptr: VoidPtr) -> VtkIdType {
        self.insert_void_pointer(self.number_of_pointers, ptr);
        self.number_of_pointers - 1
    }

    /// VTK: `vtkVoidArray::Reset`.
    pub fn reset(&mut self) {
        self.number_of_pointers = 0;
    }

    /// VTK: `vtkVoidArray::Squeeze`.
    pub fn squeeze(&mut self) {
        self.resize_and_extend(self.number_of_pointers);
    }

    /// VTK: `vtkVoidArray::GetPointer`.
    pub fn get_pointer(&mut self, id: VtkIdType) -> *mut VoidPtr {
        self.array.as_mut_ptr().wrapping_add(vtk_id_to_usize(id))
    }

    /// VTK: `vtkVoidArray::WritePointer`.
    pub fn write_pointer(&mut self, id: VtkIdType, number: VtkIdType) -> *mut VoidPtr {
        let new_size = id.saturating_add(number);
        if new_size > self.size {
            self.resize_and_extend(new_size);
        }
        self.number_of_pointers = self.number_of_pointers.max(new_size);
        self.array.as_mut_ptr().wrapping_add(vtk_id_to_usize(id))
    }

    /// VTK: `vtkVoidArray::DeepCopy`.
    pub fn deep_copy(&mut self, other: Option<&Self>) {
        let Some(other) = other else {
            return;
        };
        if std::ptr::eq(self, other) {
            return;
        }
        self.number_of_pointers = other.number_of_pointers;
        self.size = other.size;
        self.array = other.array.clone();
    }

    /// VTK: `vtkVoidArray::ResizeAndExtend`.
    fn resize_and_extend(&mut self, size: VtkIdType) -> *mut VoidPtr {
        let new_size = if size > self.size {
            self.size + size
        } else if size == self.size {
            return self.array.as_mut_ptr();
        } else {
            size
        };

        if new_size <= 0 {
            self.initialize();
            return ptr::null_mut();
        }

        let old_size = self.size;
        self.array
            .resize(vtk_id_to_usize(new_size), ptr::null_mut());
        if new_size < old_size {
            self.number_of_pointers = new_size;
        }
        self.size = new_size;
        self.array.as_mut_ptr()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkVoidArray::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkVoidArray" || Object::is_type_of(name)
    }

    /// VTK: `vtkVoidArray::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkVoidArray::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkVoidArray" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkVoidArray::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
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

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
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

impl Default for VoidArray {
    fn default() -> Self {
        Self::new()
    }
}

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkVoidArray id must be non-negative")
}
