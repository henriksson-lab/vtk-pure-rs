use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationStringValue {
    base: ObjectBase,
    value: String,
}

impl InformationStringValue {
    fn new(value: &str) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationStringValue"),
            value: value.to_owned(),
        }
    }
}

impl InformationValue for InformationStringValue {
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
        self.value.clone()
    }
}

/// VTK: `vtkInformationStringKey`.
#[derive(Debug)]
pub struct InformationStringKey {
    information_key: InformationKey,
}

impl InformationStringKey {
    /// VTK: `vtkInformationStringKey::vtkInformationStringKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationStringKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationStringKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationStringKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationStringKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<&str>) {
        let key = self.information_key.key_ptr();
        let Some(value) = value else {
            info.remove(key);
            return;
        };
        if let Some(old_value) = info
            .get_as_object_base_mut(key)
            .and_then(|value| value.as_any_mut().downcast_mut::<InformationStringValue>())
        {
            if old_value.value != value {
                old_value.value.clear();
                old_value.value.push_str(value);
                info.modified_with_key(key);
            }
        } else {
            info.set_as_object_base(key, Some(Box::new(InformationStringValue::new(value))));
        }
    }

    /// VTK: `vtkInformationStringKey::Set(const std::string&)`.
    pub fn set_string(&self, info: &mut Information, value: &str) {
        self.set(info, Some(value));
    }

    /// VTK: `vtkInformationStringKey::Get`.
    pub fn get<'a>(&self, info: &'a Information) -> Option<&'a str> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationStringValue>())
            .map(|value| value.value.as_str())
    }

    /// VTK: `vtkInformationStringKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationStringKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.get(info).unwrap_or("").to_owned()
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

    /// VTK: `vtkInformationStringKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationStringKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationStringKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationStringKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationStringKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationStringKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationStringKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
