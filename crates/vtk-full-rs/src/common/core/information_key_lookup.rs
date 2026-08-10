use std::{
    collections::BTreeMap,
    ptr,
    sync::{Mutex, OnceLock},
};

use crate::common::core::{InformationKey, Object, VtkMTimeType};

type Identifier = (String, String);

static KEYS: OnceLock<Mutex<BTreeMap<Identifier, usize>>> = OnceLock::new();

/// VTK: `vtkInformationKeyLookup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformationKeyLookup {
    object: Object,
}

impl InformationKeyLookup {
    /// VTK: `vtkInformationKeyLookup::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkInformationKeyLookup"),
        }
    }

    /// VTK: `vtkInformationKeyLookup::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = String::from("Registered Keys:\n");
        for ((location, name), key) in keys().lock().unwrap().iter() {
            let class_name = if *key == 0 {
                "(null)"
            } else {
                let key = *key as *const InformationKey;
                // The lookup map stores non-owning pointers to manager-owned
                // keys. The manager clears this map before dropping its keys.
                unsafe { (*key).get_class_name() }
            };
            output.push_str(location);
            output.push_str("::");
            output.push_str(name);
            output.push_str(" @");
            output.push_str(&format!("{:p}", *key as *const InformationKey));
            output.push_str(" (");
            output.push_str(class_name);
            output.push_str(")\n");
        }
        output
    }

    /// VTK: `vtkInformationKeyLookup::Find`.
    pub fn find(name: &str, location: &str) -> *mut InformationKey {
        keys()
            .lock()
            .unwrap()
            .get(&(location.to_owned(), name.to_owned()))
            .map_or(ptr::null_mut(), |key| *key as *mut InformationKey)
    }

    /// VTK: `vtkInformationKeyLookup::RegisterKey`.
    pub(crate) fn register_key(key: *mut InformationKey, name: &str, location: &str) {
        keys()
            .lock()
            .unwrap()
            .insert((location.to_owned(), name.to_owned()), key as usize);
    }

    /// VTK: `vtkInformationKeyLookup::Keys`.
    #[allow(dead_code)]
    pub(crate) fn keys() -> &'static Mutex<BTreeMap<Identifier, usize>> {
        keys()
    }

    pub(crate) fn clear_keys() {
        if let Some(keys) = KEYS.get() {
            keys.lock().unwrap().clear();
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkInformationKeyLookup::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationKeyLookup" || Object::is_type_of(name)
    }

    /// VTK: `vtkInformationKeyLookup::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationKeyLookup::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkInformationKeyLookup" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationKeyLookup::GetNumberOfGenerationsFromBase`.
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

impl Default for InformationKeyLookup {
    fn default() -> Self {
        Self::new()
    }
}

fn keys() -> &'static Mutex<BTreeMap<Identifier, usize>> {
    KEYS.get_or_init(|| Mutex::new(BTreeMap::new()))
}
