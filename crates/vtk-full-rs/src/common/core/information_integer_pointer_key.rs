use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationIntegerPointerValue {
    base: ObjectBase,
    value: *mut i32,
    length: i32,
}

impl InformationIntegerPointerValue {
    fn new(value: *mut i32, length: i32) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationIntegerPointerValue"),
            value,
            length,
        }
    }

    unsafe fn copy_into(&self, value: *mut i32) {
        if !self.value.is_null() && !value.is_null() && self.length > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(self.value, value, self.length as usize);
            }
        }
    }

    unsafe fn print_values(&self) -> String {
        if self.value.is_null() || self.length <= 0 {
            return String::new();
        }
        let values = unsafe { std::slice::from_raw_parts(self.value, self.length as usize) };
        values
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn print_value(&self) -> String {
        format!("int*({:p}, length={})", self.value, self.length)
    }
}

impl InformationValue for InformationIntegerPointerValue {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }

    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_value(&self, _deep: bool) -> Box<dyn InformationValue> {
        Box::new(Self::new(self.value, self.length))
    }

    fn print_value(&self) -> String {
        InformationIntegerPointerValue::print_value(self)
    }
}

/// VTK: `vtkInformationIntegerPointerKey`.
#[derive(Debug)]
pub struct InformationIntegerPointerKey {
    information_key: InformationKey,
    required_length: i32,
}

impl InformationIntegerPointerKey {
    /// VTK: `vtkInformationIntegerPointerKey::vtkInformationIntegerPointerKey`.
    pub fn new(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationIntegerPointerKey",
                name,
                location,
            ),
            required_length: length,
        })
    }

    /// VTK: `vtkInformationIntegerPointerKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationIntegerPointerKey::Set`.
    pub fn set(&self, info: &mut Information, value: *mut i32, length: i32) {
        let key = self.information_key.key_ptr();
        if value.is_null() {
            info.remove(key);
            return;
        }
        if length < 0 {
            info.remove(key);
            return;
        }
        if self.required_length >= 0 && length != self.required_length {
            info.remove(key);
            return;
        }
        info.set_as_object_base(
            key,
            Some(Box::new(InformationIntegerPointerValue::new(value, length))),
        );
    }

    /// VTK: `vtkInformationIntegerPointerKey::Get`.
    pub fn get(&self, info: &Information) -> *mut i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerPointerValue>()
            })
            .map_or(std::ptr::null_mut(), |value| value.value)
    }

    /// VTK: `vtkInformationIntegerPointerKey::Get(int*)`.
    ///
    /// # Safety
    ///
    /// The stored pointer must still be valid for `Length(info)` readable
    /// `i32` values, and `value` must be valid for that many writable values.
    pub unsafe fn get_into(&self, info: &Information, value: *mut i32) {
        if let Some(source) = info
            .get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerPointerValue>()
            })
        {
            unsafe {
                source.copy_into(value);
            }
        }
    }

    /// VTK: `vtkInformationIntegerPointerKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerPointerValue>()
            })
            .map_or(0, |value| value.length)
    }

    /// VTK: `vtkInformationIntegerPointerKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from), self.length(from));
    }

    /// VTK: `vtkInformationIntegerPointerKey::Print`.
    ///
    /// # Safety
    ///
    /// The stored pointer must still be valid for `Length(info)` readable
    /// `i32` values.
    pub unsafe fn print(&self, info: &Information) -> String {
        if !self.has(info) {
            return String::new();
        }
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerPointerValue>()
            })
            .map_or_else(String::new, |value| unsafe { value.print_values() })
    }

    /// VTK: `vtkInformationIntegerPointerKey::GetWatchAddress`.
    pub(crate) fn get_watch_address(&self, info: &mut Information) -> *mut i32 {
        self.get(info)
    }

    /// VTK: `vtkInformationKey::Has`.
    pub fn has(&self, info: &Information) -> bool {
        self.information_key.has(info)
    }

    /// VTK: `vtkInformationKey::Remove`.
    pub fn remove(&self, info: &mut Information) {
        self.information_key.remove(info);
    }

    /// VTK: `vtkInformationKey::GetName`.
    pub fn get_name(&self) -> Option<&str> {
        self.information_key.get_name()
    }

    /// VTK: `vtkInformationKey::GetLocation`.
    pub fn get_location(&self) -> Option<&str> {
        self.information_key.get_location()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.information_key.get_class_name()
    }

    /// VTK: `vtkInformationIntegerPointerKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationIntegerPointerKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerPointerKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerPointerKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationIntegerPointerKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationIntegerPointerKey::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.information_key.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.information_key.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.information_key.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.information_key.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.information_key.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.information_key.set_reference_count(reference_count);
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.information_key.get_object_description()
    }
}

impl InformationKeyRegistration for InformationIntegerPointerKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
