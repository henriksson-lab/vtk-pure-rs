use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, Variant, VtkIdType,
};

#[derive(Debug)]
struct InformationVariantValue {
    base: ObjectBase,
    value: Variant,
}

impl InformationVariantValue {
    fn new(value: Variant) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationVariantValue"),
            value,
        }
    }
}

impl InformationValue for InformationVariantValue {
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
        Box::new(Self::new(self.value.clone()))
    }

    fn print_value(&self) -> String {
        self.value.to_string()
    }
}

/// VTK: `vtkInformationVariantKey`.
#[derive(Debug)]
pub struct InformationVariantKey {
    information_key: InformationKey,
}

impl InformationVariantKey {
    /// VTK: `vtkInformationVariantKey::vtkInformationVariantKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationVariantKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationVariantKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationVariantKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationVariantKey::Set`.
    pub fn set(&self, info: &mut Information, value: Variant) {
        let key = self.information_key.key_ptr();
        if let Some(old_value) = info
            .get_as_object_base_mut(key)
            .and_then(|value| value.as_any_mut().downcast_mut::<InformationVariantValue>())
        {
            if old_value.value != value {
                old_value.value = value;
                info.modified_with_key(key);
            }
        } else {
            info.set_as_object_base(key, Some(Box::new(InformationVariantValue::new(value))));
        }
    }

    /// VTK: `vtkInformationVariantKey::Get`.
    pub fn get<'a>(&self, info: &'a Information) -> &'a Variant {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationVariantValue>())
            .map_or(&Variant::Invalid, |value| &value.value)
    }

    /// VTK: `vtkInformationVariantKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        if self.has(from) {
            self.set(to, self.get(from).clone());
        } else {
            to.remove(self.information_key.key_ptr());
        }
    }

    /// VTK: `vtkInformationVariantKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            self.get(info).to_string()
        } else {
            String::new()
        }
    }

    /// VTK: `vtkInformationVariantKey::GetWatchAddress`.
    pub(crate) fn get_watch_address(&self, info: &mut Information) -> *mut Variant {
        info.get_as_object_base_mut(self.information_key.key_ptr())
            .and_then(|value| value.as_any_mut().downcast_mut::<InformationVariantValue>())
            .map_or(std::ptr::null_mut(), |value| &mut value.value)
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

    /// VTK: `vtkInformationVariantKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationVariantKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationVariantKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationVariantKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationVariantKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationVariantKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationVariantKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
