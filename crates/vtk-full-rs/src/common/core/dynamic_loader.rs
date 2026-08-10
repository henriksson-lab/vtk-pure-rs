use std::ffi::{c_char, c_void};

use super::{
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

pub type VtkLibHandle = *mut c_void;
pub type VtkSymbolPointer = *mut c_void;

/// VTK: `vtkDynamicLoader`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicLoader {
    object: Object,
}

impl DynamicLoader {
    /// VTK: `vtkDynamicLoader::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDynamicLoader"),
        }
    }

    /// VTK: `vtkDynamicLoader::OpenLibrary(const char*)`.
    pub fn open_library(libname: *const c_char) -> VtkLibHandle {
        open_library_impl(libname, default_open_flags())
    }

    /// VTK: `vtkDynamicLoader::OpenLibrary(const char*, int)`.
    pub fn open_library_with_flags(libname: *const c_char, flags: i32) -> VtkLibHandle {
        open_library_impl(libname, flags)
    }

    /// VTK: `vtkDynamicLoader::CloseLibrary`.
    pub fn close_library(lib: VtkLibHandle) -> i32 {
        close_library_impl(lib)
    }

    /// VTK: `vtkDynamicLoader::GetSymbolAddress`.
    pub fn get_symbol_address(lib: VtkLibHandle, sym: *const c_char) -> VtkSymbolPointer {
        get_symbol_address_impl(lib, sym)
    }

    /// VTK: `vtkDynamicLoader::LibPrefix`.
    pub fn lib_prefix() -> *const c_char {
        lib_prefix_impl()
    }

    /// VTK: `vtkDynamicLoader::LibExtension`.
    pub fn lib_extension() -> *const c_char {
        lib_extension_impl()
    }

    /// VTK: `vtkDynamicLoader::LastError`.
    pub fn last_error() -> *const c_char {
        last_error_impl()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkDynamicLoader::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkDynamicLoader" || Object::is_type_of(name)
    }

    /// VTK: `vtkDynamicLoader::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkDynamicLoader::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkDynamicLoader" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkDynamicLoader::GetNumberOfGenerationsFromBase`.
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

impl Default for DynamicLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(unix)]
fn default_open_flags() -> i32 {
    libc::RTLD_LAZY
}

#[cfg(not(unix))]
fn default_open_flags() -> i32 {
    0
}

#[cfg(unix)]
fn open_library_impl(libname: *const c_char, flags: i32) -> VtkLibHandle {
    unsafe { libc::dlopen(libname, flags) }
}

#[cfg(not(unix))]
fn open_library_impl(_libname: *const c_char, _flags: i32) -> VtkLibHandle {
    std::ptr::null_mut()
}

#[cfg(unix)]
fn close_library_impl(lib: VtkLibHandle) -> i32 {
    unsafe { libc::dlclose(lib) }
}

#[cfg(not(unix))]
fn close_library_impl(_lib: VtkLibHandle) -> i32 {
    0
}

#[cfg(unix)]
fn get_symbol_address_impl(lib: VtkLibHandle, sym: *const c_char) -> VtkSymbolPointer {
    unsafe { libc::dlsym(lib, sym) }
}

#[cfg(not(unix))]
fn get_symbol_address_impl(_lib: VtkLibHandle, _sym: *const c_char) -> VtkSymbolPointer {
    std::ptr::null_mut()
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn lib_prefix_impl() -> *const c_char {
    b"lib\0".as_ptr().cast()
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
fn lib_prefix_impl() -> *const c_char {
    b"lib\0".as_ptr().cast()
}

#[cfg(not(unix))]
fn lib_prefix_impl() -> *const c_char {
    b"\0".as_ptr().cast()
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn lib_extension_impl() -> *const c_char {
    b".dylib\0".as_ptr().cast()
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
fn lib_extension_impl() -> *const c_char {
    b".so\0".as_ptr().cast()
}

#[cfg(not(unix))]
fn lib_extension_impl() -> *const c_char {
    b".dll\0".as_ptr().cast()
}

#[cfg(unix)]
fn last_error_impl() -> *const c_char {
    unsafe { libc::dlerror() }
}

#[cfg(not(unix))]
fn last_error_impl() -> *const c_char {
    std::ptr::null()
}
