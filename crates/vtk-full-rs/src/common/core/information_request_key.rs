use crate::common::core::{
    information_key::InformationKeyRegistration, CommonInformationKeyManager, Information,
    InformationKey, VtkIdType,
};

/// VTK: `vtkInformationRequestKey`.
#[derive(Debug)]
pub struct InformationRequestKey {
    information_key: InformationKey,
}

impl InformationRequestKey {
    /// VTK: `vtkInformationRequestKey::vtkInformationRequestKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationRequestKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationRequestKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationRequestKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationRequestKey::Set`.
    pub fn set(&self, info: &mut Information) {
        let key = self.information_key.key_ptr();
        if info.get_request() != key {
            info.set_request(key);
            info.modified_with_key(key);
        }
    }

    /// VTK: `vtkInformationRequestKey::Has`.
    pub fn has(&self, info: &Information) -> bool {
        info.get_request() == self.information_key.key_ptr()
    }

    /// VTK: `vtkInformationRequestKey::Remove`.
    pub fn remove(&self, info: &mut Information) {
        info.set_request(std::ptr::null_mut());
    }

    /// VTK: `vtkInformationRequestKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        to.set_request(from.get_request());
    }

    /// VTK: `vtkInformationRequestKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            "1\n".to_owned()
        } else {
            String::new()
        }
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

    /// VTK: `vtkInformationRequestKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationRequestKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationRequestKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationRequestKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationRequestKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationRequestKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationRequestKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
