use crate::common::core::{Information, ObjectBase, VtkIdType};

pub(crate) trait InformationKeyRegistration: Send {
    fn information_key(&self) -> &InformationKey;
    fn information_key_mut(&mut self) -> &mut InformationKey;
}

/// VTK: `vtkInformationKey`.
#[derive(Debug)]
pub struct InformationKey {
    base: ObjectBase,
    name: Option<String>,
    location: Option<String>,
}

impl InformationKey {
    /// VTK: `vtkInformationKey::vtkInformationKey`.
    #[allow(dead_code)]
    pub(crate) fn new(name: Option<&str>, location: Option<&str>) -> Self {
        Self::with_class_name("vtkInformationKey", name, location)
    }

    pub(crate) fn with_class_name(
        class_name: &'static str,
        name: Option<&str>,
        location: Option<&str>,
    ) -> Self {
        Self {
            base: ObjectBase::with_class_name(class_name),
            name: name.map(str::to_owned),
            location: location.map(str::to_owned),
        }
    }

    pub(crate) fn key_ptr(&self) -> *mut InformationKey {
        self as *const Self as *mut Self
    }

    /// VTK: `vtkInformationKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.base.get_class_name().to_string()
    }

    /// VTK: `vtkInformationKey::GetName`.
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// VTK: `vtkInformationKey::GetLocation`.
    pub fn get_location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// VTK: `vtkInformationKey::Has`.
    pub fn has(&self, info: &Information) -> bool {
        info.has(self.key_ptr()) != 0
    }

    /// VTK: `vtkInformationKey::Remove`.
    pub fn remove(&self, info: &mut Information) {
        info.remove(self.key_ptr());
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.base.get_class_name()
    }

    /// VTK: `vtkInformationKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationKey" || ObjectBase::is_type_of(name)
    }

    /// VTK: `vtkInformationKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationKey" => 0,
            "vtkObjectBase" => 1,
            _ => ObjectBase::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationKey::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.base.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.base.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.base.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.base.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.base.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.base.set_reference_count(reference_count);
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.base.get_object_description()
    }
}

impl InformationKeyRegistration for InformationKey {
    fn information_key(&self) -> &InformationKey {
        self
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        self
    }
}
