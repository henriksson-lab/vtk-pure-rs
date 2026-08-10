use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationStringVectorValue {
    base: ObjectBase,
    value: Vec<String>,
}

impl InformationStringVectorValue {
    fn new() -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationStringVectorValue"),
            value: Vec::new(),
        }
    }
}

impl InformationValue for InformationStringVectorValue {
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
        Box::new(Self {
            base: ObjectBase::with_class_name("vtkInformationStringVectorValue"),
            value: self.value.clone(),
        })
    }

    fn print_value(&self) -> String {
        self.value.join(" ")
    }
}

/// VTK: `vtkInformationStringVectorKey`.
#[derive(Debug)]
pub struct InformationStringVectorKey {
    information_key: InformationKey,
    required_length: i32,
}

impl InformationStringVectorKey {
    /// VTK: `vtkInformationStringVectorKey::vtkInformationStringVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationStringVectorKey",
                name,
                location,
            ),
            required_length: length,
        })
    }

    /// VTK: `vtkInformationStringVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        Self::new(name, location, length)
    }

    /// VTK: `vtkInformationStringVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationStringVectorKey::Append(const char*)`.
    pub fn append(&self, info: &mut Information, value: &str) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationStringVectorValue>()
        }) {
            vector.value.push(value.to_owned());
            info.modified_with_key(key);
        } else {
            self.set(info, value, 0);
        }
    }

    /// VTK: `vtkInformationStringVectorKey::Set(const char*)`.
    pub fn set(&self, info: &mut Information, value: &str, index: i32) {
        if index < 0 {
            return;
        }
        let index = index as usize;
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationStringVectorValue>()
        }) {
            if vector.value.len() <= index || vector.value[index] != value {
                while vector.value.len() <= index {
                    vector.value.push(String::new());
                }
                vector.value[index].clear();
                vector.value[index].push_str(value);
                info.modified_with_key(key);
            }
        } else {
            let mut vector = InformationStringVectorValue::new();
            while vector.value.len() <= index {
                vector.value.push(String::new());
            }
            vector.value[index].push_str(value);
            info.set_as_object_base(key, Some(Box::new(vector)));
        }
    }

    /// VTK: `vtkInformationStringVectorKey::Append(const std::string&)`.
    pub fn append_string(&self, info: &mut Information, value: &str) {
        self.append(info, value);
    }

    /// VTK: `vtkInformationStringVectorKey::Set(const std::string&)`.
    pub fn set_string(&self, info: &mut Information, value: &str, index: i32) {
        self.set(info, value, index);
    }

    /// VTK: `vtkInformationStringVectorKey::Get`.
    pub fn get<'a>(&self, info: &'a Information, index: i32) -> Option<&'a str> {
        if index < 0 {
            return None;
        }
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationStringVectorValue>()
            })
            .and_then(|value| value.value.get(index as usize))
            .map(String::as_str)
    }

    /// VTK: `vtkInformationStringVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationStringVectorValue>()
            })
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationStringVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        let length = self.length(from);
        for index in 0..length {
            if let Some(value) = self.get(from, index) {
                self.set(to, value, index);
            }
        }
    }

    /// VTK: `vtkInformationStringVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            (0..self.length(info))
                .filter_map(|index| self.get(info, index))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
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

    /// VTK: `vtkInformationStringVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationStringVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationStringVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationStringVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationStringVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationStringVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationStringVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
