use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationIdTypeValue {
    base: ObjectBase,
    value: VtkIdType,
}

impl InformationIdTypeValue {
    fn new(value: VtkIdType) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationIdTypeValue"),
            value,
        }
    }
}

impl InformationValue for InformationIdTypeValue {
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
        Box::new(Self::new(self.value))
    }

    fn print_value(&self) -> String {
        self.value.to_string()
    }
}

/// VTK: `vtkInformationIdTypeKey`.
#[derive(Debug)]
pub struct InformationIdTypeKey {
    information_key: InformationKey,
}

impl InformationIdTypeKey {
    /// VTK: `vtkInformationIdTypeKey::vtkInformationIdTypeKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationIdTypeKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationIdTypeKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationIdTypeKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationIdTypeKey::Set`.
    pub fn set(&self, info: &mut Information, value: VtkIdType) {
        let key = self.information_key.key_ptr();
        if let Some(old_value) = info
            .get_as_object_base_mut(key)
            .and_then(|value| value.as_any_mut().downcast_mut::<InformationIdTypeValue>())
        {
            if old_value.value != value {
                old_value.value = value;
                info.modified_with_key(key);
            }
        } else {
            info.set_as_object_base(key, Some(Box::new(InformationIdTypeValue::new(value))));
        }
    }

    /// VTK: `vtkInformationIdTypeKey::Get`.
    pub fn get(&self, info: &Information) -> VtkIdType {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationIdTypeValue>())
            .map_or(0, |value| value.value)
    }

    /// VTK: `vtkInformationIdTypeKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        if self.has(from) {
            self.set(to, self.get(from));
        } else {
            to.remove(self.information_key.key_ptr());
        }
    }

    /// VTK: `vtkInformationIdTypeKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            self.get(info).to_string()
        } else {
            String::new()
        }
    }

    /// VTK: `vtkInformationIdTypeKey::GetWatchAddress`.
    pub fn get_watch_address(&self, info: &mut Information) -> *mut VtkIdType {
        info.get_as_object_base_mut(self.information_key.key_ptr())
            .and_then(|value| value.as_any_mut().downcast_mut::<InformationIdTypeValue>())
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

    /// VTK: `vtkInformationIdTypeKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationIdTypeKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationIdTypeKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationIdTypeKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationIdTypeKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationIdTypeKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationIdTypeKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
