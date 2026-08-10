use std::{any::Any, cell::RefCell, collections::HashMap, ptr::NonNull, rc::Rc};

use crate::common::core::{InformationKey, Object, ObjectBase, VtkMTimeType};

pub type InformationHandle = Rc<RefCell<Information>>;

pub(crate) trait InformationValue: Any + std::fmt::Debug {
    fn object_base(&self) -> &ObjectBase;
    fn object_base_mut(&mut self) -> &mut ObjectBase;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_value(&self, deep: bool) -> Box<dyn InformationValue>;
    fn print_value(&self) -> String;
}

/// VTK: `vtkInformation`.
#[derive(Debug)]
pub struct Information {
    object: Object,
    map: HashMap<NonNull<InformationKey>, Box<dyn InformationValue>>,
    request: *mut InformationKey,
}

impl Information {
    /// VTK: `vtkInformation::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkInformation"),
            map: HashMap::with_capacity(33),
            request: std::ptr::null_mut(),
        }
    }

    /// VTK: `vtkInformation::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        if let Some(request) = NonNull::new(self.request) {
            let request = unsafe { request.as_ref() };
            if let Some(name) = request.get_name() {
                output.push_str("\nRequest: ");
                output.push_str(name);
            }
        }
        let keys = self.print_keys();
        if !keys.is_empty() {
            output.push('\n');
            output.push_str(&keys);
        }
        output
    }

    /// VTK: `vtkInformation::PrintKeys`.
    pub fn print_keys(&self) -> String {
        let mut entries = Vec::with_capacity(self.map.len());
        for (key, value) in &self.map {
            let key = unsafe { key.as_ref() };
            let name = key.get_name().unwrap_or("");
            entries.push(format!("{name}: {}", value.print_value()));
        }
        entries.sort();
        entries.join("\n")
    }

    /// VTK: `vtkInformation::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkInformation::Modified(vtkInformationKey*)`.
    pub fn modified_with_key(&mut self, _key: *mut InformationKey) {
        self.object.modified();
    }

    /// VTK: `vtkInformation::Clear`.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// VTK: `vtkInformation::GetNumberOfKeys`.
    pub fn get_number_of_keys(&self) -> i32 {
        self.map.len() as i32
    }

    /// VTK: `vtkInformation::Copy`.
    pub fn copy(&mut self, from: Option<&Information>, deep: bool) {
        self.map.clear();
        if let Some(from) = from {
            self.append(Some(from), deep);
        }
    }

    /// VTK: `vtkInformation::Append`.
    pub fn append(&mut self, from: Option<&Information>, deep: bool) {
        let Some(from) = from else {
            return;
        };
        for key in from.map.keys().copied().collect::<Vec<_>>() {
            self.copy_entry(from, key.as_ptr(), deep);
        }
    }

    /// VTK: `vtkInformation::CopyEntry(vtkInformation*, vtkInformationKey*)`.
    pub fn copy_entry(&mut self, from: &Information, key: *mut InformationKey, deep: bool) {
        let Some(key) = NonNull::new(key) else {
            return;
        };
        if let Some(value) = from.map.get(&key) {
            self.set_as_object_base(key.as_ptr(), Some(value.clone_value(deep)));
        } else {
            self.set_as_object_base(key.as_ptr(), None);
        }
    }

    /// VTK: `vtkInformation::Has(vtkInformationKey*)`.
    pub fn has(&self, key: *mut InformationKey) -> i32 {
        self.get_as_object_base(key).is_some() as i32
    }

    /// VTK: `vtkInformation::Remove(vtkInformationKey*)`.
    pub fn remove(&mut self, key: *mut InformationKey) {
        self.set_as_object_base(key, None);
    }

    /// VTK: `vtkInformation::SetRequest`.
    pub fn set_request(&mut self, request: *mut InformationKey) {
        self.request = request;
    }

    /// VTK: `vtkInformation::GetRequest`.
    pub fn get_request(&self) -> *mut InformationKey {
        self.request
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkInformation::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformation" || Object::is_type_of(name)
    }

    /// VTK: `vtkInformation::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformation::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkInformation" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformation::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
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

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }

    pub(crate) fn set_as_object_base(
        &mut self,
        key: *mut InformationKey,
        value: Option<Box<dyn InformationValue>>,
    ) {
        let Some(key) = NonNull::new(key) else {
            return;
        };
        match value {
            Some(value) => {
                self.map.insert(key, value);
            }
            None => {
                self.map.remove(&key);
            }
        }
        self.modified_with_key(key.as_ptr());
    }

    pub(crate) fn get_as_object_base(
        &self,
        key: *mut InformationKey,
    ) -> Option<&dyn InformationValue> {
        NonNull::new(key).and_then(|key| self.map.get(&key).map(Box::as_ref))
    }

    pub(crate) fn get_as_object_base_mut(
        &mut self,
        key: *mut InformationKey,
    ) -> Option<&mut dyn InformationValue> {
        let key = NonNull::new(key)?;
        self.map.get_mut(&key).map(Box::as_mut)
    }

    pub(crate) fn key_ptrs(&self) -> Vec<*mut InformationKey> {
        self.map.keys().map(|key| key.as_ptr()).collect()
    }
}

impl Default for Information {
    fn default() -> Self {
        Self::new()
    }
}
