use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationDoubleVectorValue {
    base: ObjectBase,
    value: Vec<f64>,
}

impl InformationDoubleVectorValue {
    fn new(value: &[f64]) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationDoubleVectorValue"),
            value: value.to_vec(),
        }
    }
}

impl InformationValue for InformationDoubleVectorValue {
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
            .map(f64::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// VTK: `vtkInformationDoubleVectorKey`.
#[derive(Debug)]
pub struct InformationDoubleVectorKey {
    information_key: InformationKey,
    required_length: i32,
}

impl InformationDoubleVectorKey {
    /// VTK: `vtkInformationDoubleVectorKey::vtkInformationDoubleVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationDoubleVectorKey",
                name,
                location,
            ),
            required_length: length,
        })
    }

    /// VTK: `vtkInformationDoubleVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        Self::new(name, location, length)
    }

    /// VTK: `vtkInformationDoubleVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationDoubleVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: f64) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationDoubleVectorValue>()
        }) {
            vector.value.push(value);
            info.modified_with_key(key);
        } else {
            self.set(info, Some(&[value]));
        }
    }

    /// VTK: `vtkInformationDoubleVectorKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<&[f64]>) {
        let key = self.information_key.key_ptr();
        let Some(value) = value else {
            info.remove(key);
            return;
        };
        if self.required_length >= 0 && value.len() as i32 != self.required_length {
            info.remove(key);
            return;
        }
        info.set_as_object_base(
            key,
            Some(Box::new(InformationDoubleVectorValue::new(value))),
        );
    }

    /// VTK: `vtkInformationDoubleVectorKey::Get()`.
    pub fn get<'a>(&self, info: &'a Information) -> Option<&'a [f64]> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationDoubleVectorValue>()
            })
            .and_then(|value| {
                if value.value.is_empty() {
                    None
                } else {
                    Some(value.value.as_slice())
                }
            })
    }

    /// VTK: `vtkInformationDoubleVectorKey::Get(int)`.
    pub fn get_value(&self, info: &Information, index: i32) -> f64 {
        if index < 0 || index >= self.length(info) {
            return 0.0;
        }
        self.get(info).map_or(0.0, |value| value[index as usize])
    }

    /// VTK: `vtkInformationDoubleVectorKey::Get(double*)`.
    pub fn get_into(&self, info: &Information, value: &mut [f64]) {
        if let Some(source) = info
            .get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationDoubleVectorValue>()
            })
        {
            let count = value.len().min(source.value.len());
            value[..count].copy_from_slice(&source.value[..count]);
        }
    }

    /// VTK: `vtkInformationDoubleVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationDoubleVectorValue>()
            })
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationDoubleVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationDoubleVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            info.get_as_object_base(self.information_key.key_ptr())
                .and_then(|value| {
                    value
                        .as_any()
                        .downcast_ref::<InformationDoubleVectorValue>()
                })
                .map_or_else(String::new, InformationDoubleVectorValue::print_value)
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

    /// VTK: `vtkInformationDoubleVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationDoubleVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationDoubleVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationDoubleVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationDoubleVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationDoubleVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationDoubleVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
