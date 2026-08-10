use crate::common::{
    core::{
        information_key::InformationKeyRegistration, CommonInformationKeyManager, Information,
        InformationDataObjectKey, InformationKey, VtkIdType,
    },
    data_model::DataObjectHandle,
    execution_model::DemandDrivenPipeline,
};

/// VTK: `vtkInformationDataObjectMetaDataKey`.
#[derive(Debug)]
pub struct InformationDataObjectMetaDataKey {
    data_object_key: InformationDataObjectKey,
}

impl InformationDataObjectMetaDataKey {
    /// VTK: `vtkInformationDataObjectMetaDataKey::vtkInformationDataObjectMetaDataKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            data_object_key: InformationDataObjectKey::with_class_name(
                "vtkInformationDataObjectMetaDataKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.data_object_key.print_self()
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::CopyDefaultInformation`.
    pub fn copy_default_information(
        &self,
        request: &Information,
        from_info: &Information,
        to_info: &mut Information,
    ) {
        if DemandDrivenPipeline::request_information().has(request) {
            self.shallow_copy(from_info, to_info);
        }
    }

    /// VTK: `vtkInformationDataObjectKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<DataObjectHandle>) {
        self.data_object_key.set(info, value);
    }

    /// VTK: `vtkInformationDataObjectKey::Get`.
    pub fn get(&self, info: &Information) -> Option<DataObjectHandle> {
        self.data_object_key.get(info)
    }

    /// VTK: `vtkInformationDataObjectKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.data_object_key.shallow_copy(from, to);
    }

    /// VTK: `vtkInformationDataObjectKey::Report`.
    pub fn report(&self, info: &Information) {
        self.data_object_key.report(info);
    }

    /// VTK: `vtkInformationKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.data_object_key.print(info)
    }

    /// VTK: `vtkInformationKey::Has`.
    pub fn has(&self, info: &Information) -> bool {
        self.data_object_key.has(info)
    }

    /// VTK: `vtkInformationKey::Remove`.
    pub fn remove(&self, info: &mut Information) {
        self.data_object_key.remove(info);
    }

    /// VTK: `vtkInformationKey::GetName`.
    pub fn get_name(&self) -> Option<&str> {
        self.data_object_key.get_name()
    }

    /// VTK: `vtkInformationKey::GetLocation`.
    pub fn get_location(&self) -> Option<&str> {
        self.data_object_key.get_location()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.data_object_key.get_class_name()
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationDataObjectMetaDataKey" || InformationDataObjectKey::is_type_of(name)
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationDataObjectMetaDataKey" => 0,
            "vtkInformationDataObjectKey" => 1,
            "vtkInformationKey" => 2,
            "vtkObjectBase" => 3,
            _ => InformationDataObjectKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationDataObjectMetaDataKey::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.data_object_key.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.data_object_key.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.data_object_key.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.data_object_key.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.data_object_key.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.data_object_key.set_reference_count(reference_count);
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.data_object_key.get_object_description()
    }
}

impl InformationKeyRegistration for InformationDataObjectMetaDataKey {
    fn information_key(&self) -> &InformationKey {
        self.data_object_key.information_key()
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        self.data_object_key.information_key_mut()
    }
}
