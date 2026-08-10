use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationIntegerVectorValue {
    base: ObjectBase,
    value: Vec<i32>,
}

impl InformationIntegerVectorValue {
    fn new(value: &[i32]) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationIntegerVectorValue"),
            value: value.to_vec(),
        }
    }
}

impl InformationValue for InformationIntegerVectorValue {
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
        Box::new(Self::new(&self.value))
    }

    fn print_value(&self) -> String {
        self.value
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// VTK: `vtkInformationIntegerVectorKey`.
#[derive(Debug)]
pub struct InformationIntegerVectorKey {
    information_key: InformationKey,
    required_length: i32,
}

impl InformationIntegerVectorKey {
    /// VTK: `vtkInformationIntegerVectorKey::vtkInformationIntegerVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationIntegerVectorKey",
                name,
                location,
            ),
            required_length: length,
        })
    }

    /// VTK: `vtkInformationIntegerVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        Self::new(name, location, length)
    }

    /// VTK: `vtkInformationIntegerVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationIntegerVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: i32) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationIntegerVectorValue>()
        }) {
            vector.value.push(value);
            info.modified_with_key(key);
        } else {
            self.set(info, Some(&[value]));
        }
    }

    /// VTK: `vtkInformationIntegerVectorKey::Set(vtkInformation*)`.
    pub fn set_empty(&self, info: &mut Information) {
        self.set(info, Some(&[]));
    }

    /// VTK: `vtkInformationIntegerVectorKey::Set(const int*, int)`.
    pub fn set(&self, info: &mut Information, value: Option<&[i32]>) {
        let key = self.information_key.key_ptr();
        let Some(value) = value else {
            info.remove(key);
            return;
        };
        if self.required_length >= 0 && value.len() as i32 != self.required_length {
            info.remove(key);
            return;
        }
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationIntegerVectorValue>()
        }) {
            if vector.value.len() == value.len() {
                vector.value.copy_from_slice(value);
                info.modified_with_key(key);
            } else {
                info.set_as_object_base(
                    key,
                    Some(Box::new(InformationIntegerVectorValue::new(value))),
                );
            }
        } else {
            info.set_as_object_base(
                key,
                Some(Box::new(InformationIntegerVectorValue::new(value))),
            );
        }
    }

    /// VTK: `vtkInformationIntegerVectorKey::Get()`.
    pub fn get<'a>(&self, info: &'a Information) -> Option<&'a [i32]> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerVectorValue>()
            })
            .and_then(|value| {
                if value.value.is_empty() {
                    None
                } else {
                    Some(value.value.as_slice())
                }
            })
    }

    /// VTK: `vtkInformationIntegerVectorKey::Get(int)`.
    pub fn get_value(&self, info: &Information, index: i32) -> i32 {
        if index < 0 || index >= self.length(info) {
            return 0;
        }
        self.get(info).map_or(0, |value| value[index as usize])
    }

    /// VTK: `vtkInformationIntegerVectorKey::Get(int*)`.
    pub fn get_into(&self, info: &Information, value: &mut [i32]) {
        if let Some(source) = info
            .get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerVectorValue>()
            })
        {
            let count = value.len().min(source.value.len());
            value[..count].copy_from_slice(&source.value[..count]);
        }
    }

    /// VTK: `vtkInformationIntegerVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationIntegerVectorValue>()
            })
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationIntegerVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationIntegerVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            info.get_as_object_base(self.information_key.key_ptr())
                .and_then(|value| {
                    value
                        .as_any()
                        .downcast_ref::<InformationIntegerVectorValue>()
                })
                .map_or_else(String::new, InformationIntegerVectorValue::print_value)
        } else {
            String::new()
        }
    }

    /// VTK: `vtkInformationIntegerVectorKey::GetWatchAddress`.
    pub(crate) fn get_watch_address(&self, info: &mut Information) -> *mut i32 {
        info.get_as_object_base_mut(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any_mut()
                    .downcast_mut::<InformationIntegerVectorValue>()
            })
            .and_then(|value| {
                if value.value.is_empty() {
                    None
                } else {
                    Some(value.value.as_mut_ptr())
                }
            })
            .unwrap_or(std::ptr::null_mut())
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

    /// VTK: `vtkInformationIntegerVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationIntegerVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationIntegerVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationIntegerVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationIntegerVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
