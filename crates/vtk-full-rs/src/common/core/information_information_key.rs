use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationHandle, InformationKey, ObjectBase,
    VtkIdType,
};

#[derive(Debug)]
struct InformationInformationValue {
    base: ObjectBase,
    value: InformationHandle,
}

impl InformationInformationValue {
    fn new(value: InformationHandle) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationInformationValue"),
            value,
        }
    }
}

impl InformationValue for InformationInformationValue {
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

    fn clone_value(&self, deep: bool) -> Box<dyn InformationValue> {
        if deep {
            let mut copy = Information::new();
            copy.copy(Some(&self.value.borrow()), true);
            Box::new(Self::new(std::rc::Rc::new(std::cell::RefCell::new(copy))))
        } else {
            Box::new(Self::new(self.value.clone()))
        }
    }

    fn print_value(&self) -> String {
        self.value.borrow().get_object_description()
    }
}

/// VTK: `vtkInformationInformationKey`.
#[derive(Debug)]
pub struct InformationInformationKey {
    information_key: InformationKey,
}

impl InformationInformationKey {
    /// VTK: `vtkInformationInformationKey::vtkInformationInformationKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationInformationKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationInformationKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationInformationKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationInformationKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<InformationHandle>) {
        let key = self.information_key.key_ptr();
        if let Some(value) = value {
            info.set_as_object_base(key, Some(Box::new(InformationInformationValue::new(value))));
        } else {
            info.remove(key);
        }
    }

    /// VTK: `vtkInformationInformationKey::Get`.
    pub fn get(&self, info: &Information) -> Option<InformationHandle> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationInformationValue>())
            .map(|value| value.value.clone())
    }

    /// VTK: `vtkInformationInformationKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationInformationKey::DeepCopy`.
    pub fn deep_copy(&self, from: &Information, to: &mut Information) {
        if let Some(from_info) = self.get(from) {
            let mut to_info = Information::new();
            to_info.copy(Some(&from_info.borrow()), true);
            self.set(to, Some(std::rc::Rc::new(std::cell::RefCell::new(to_info))));
        } else {
            self.set(to, None);
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

    /// VTK: `vtkInformationInformationKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationInformationKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationInformationKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationInformationKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationInformationKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationInformationKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationInformationKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
