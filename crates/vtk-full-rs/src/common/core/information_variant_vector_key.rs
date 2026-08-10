use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, Variant, VtkIdType,
};

#[derive(Debug)]
struct InformationVariantVectorValue {
    base: ObjectBase,
    value: Vec<Variant>,
}

impl InformationVariantVectorValue {
    fn new(value: &[Variant]) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationVariantVectorValue"),
            value: value.to_vec(),
        }
    }
}

impl InformationValue for InformationVariantVectorValue {
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
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// VTK: `vtkInformationVariantVectorKey`.
#[derive(Debug)]
pub struct InformationVariantVectorKey {
    information_key: InformationKey,
    required_length: i32,
}

impl InformationVariantVectorKey {
    /// VTK: `vtkInformationVariantVectorKey::vtkInformationVariantVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationVariantVectorKey",
                name,
                location,
            ),
            required_length: length,
        })
    }

    /// VTK: `vtkInformationVariantVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>, length: i32) -> *mut Self {
        Self::new(name, location, length)
    }

    /// VTK: `vtkInformationVariantVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationVariantVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: Variant) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationVariantVectorValue>()
        }) {
            vector.value.push(value);
            info.modified_with_key(key);
        } else {
            self.set(info, Some(&[value]));
        }
    }

    /// VTK: `vtkInformationVariantVectorKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<&[Variant]>) {
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
            Some(Box::new(InformationVariantVectorValue::new(value))),
        );
    }

    /// VTK: `vtkInformationVariantVectorKey::Get()`.
    pub fn get<'a>(&self, info: &'a Information) -> Option<&'a [Variant]> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationVariantVectorValue>()
            })
            .and_then(|value| {
                if value.value.is_empty() {
                    None
                } else {
                    Some(value.value.as_slice())
                }
            })
    }

    /// VTK: `vtkInformationVariantVectorKey::Get(int)`.
    pub fn get_value<'a>(&self, info: &'a Information, index: i32) -> &'a Variant {
        if index < 0 || index >= self.length(info) {
            return &Variant::Invalid;
        }
        self.get(info)
            .map_or(&Variant::Invalid, |value| &value[index as usize])
    }

    /// VTK: `vtkInformationVariantVectorKey::Get(vtkVariant*)`.
    pub fn get_into(&self, info: &Information, value: &mut [Variant]) {
        if let Some(source) = info
            .get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationVariantVectorValue>()
            })
        {
            let count = value.len().min(source.value.len());
            value[..count].clone_from_slice(&source.value[..count]);
        }
    }

    /// VTK: `vtkInformationVariantVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationVariantVectorValue>()
            })
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationVariantVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationVariantVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            info.get_as_object_base(self.information_key.key_ptr())
                .and_then(|value| {
                    value
                        .as_any()
                        .downcast_ref::<InformationVariantVectorValue>()
                })
                .map_or_else(String::new, InformationVariantVectorValue::print_value)
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

    /// VTK: `vtkInformationVariantVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationVariantVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationVariantVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationVariantVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationVariantVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationVariantVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationVariantVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
