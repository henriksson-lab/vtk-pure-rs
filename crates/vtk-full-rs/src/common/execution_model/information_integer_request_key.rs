use crate::common::{
    core::{
        information_key::InformationKeyRegistration, CommonInformationKeyManager, Information,
        InformationIntegerKey, InformationKey, VtkIdType,
    },
    execution_model::StreamingDemandDrivenPipeline,
};

/// VTK: `vtkInformationIntegerRequestKey`.
#[derive(Debug)]
pub struct InformationIntegerRequestKey {
    integer_key: InformationIntegerKey,
    pub(crate) data_key: Option<usize>,
}

impl InformationIntegerRequestKey {
    /// VTK: `vtkInformationIntegerRequestKey::vtkInformationIntegerRequestKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            integer_key: InformationIntegerKey::with_class_name(
                "vtkInformationIntegerRequestKey",
                name,
                location,
            ),
            data_key: None,
        })
    }

    /// VTK: `vtkInformationIntegerRequestKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationIntegerRequestKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.integer_key.print_self()
    }

    /// VTK: `vtkInformationIntegerRequestKey::NeedToExecute`.
    pub fn need_to_execute(&self, pipeline_info: &Information, dobj_info: &Information) -> bool {
        let data_key = self.data_key();
        !data_key.has(dobj_info) || data_key.get(dobj_info) != self.get(pipeline_info)
    }

    /// VTK: `vtkInformationIntegerRequestKey::StoreMetaData`.
    pub fn store_meta_data(
        &self,
        _request: &Information,
        pipeline_info: &Information,
        dobj_info: &mut Information,
    ) {
        self.data_key().set(dobj_info, self.get(pipeline_info));
    }

    /// VTK: `vtkInformationIntegerRequestKey::CopyDefaultInformation`.
    pub fn copy_default_information(
        &self,
        request: &Information,
        from_info: &Information,
        to_info: &mut Information,
    ) {
        if StreamingDemandDrivenPipeline::request_update_extent().has(request) {
            self.shallow_copy(from_info, to_info);
        }
    }

    pub(crate) fn data_key(&self) -> &InformationIntegerKey {
        let data_key = self
            .data_key
            .expect("vtkInformationIntegerRequestKey DataKey must be set by a subclass");
        unsafe { &*(data_key as *const InformationIntegerKey) }
    }

    /// VTK: `vtkInformationIntegerKey::Set`.
    pub fn set(&self, info: &mut Information, value: i32) {
        self.integer_key.set(info, value);
    }

    /// VTK: `vtkInformationIntegerKey::Get`.
    pub fn get(&self, info: &Information) -> i32 {
        self.integer_key.get(info)
    }

    /// VTK: `vtkInformationIntegerKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.integer_key.shallow_copy(from, to);
    }

    /// VTK: `vtkInformationIntegerKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.integer_key.print(info)
    }

    /// VTK: `vtkInformationIntegerKey::GetWatchAddress`.
    pub fn get_watch_address(&self, info: &mut Information) -> *mut i32 {
        self.integer_key.get_watch_address(info)
    }

    /// VTK: `vtkInformationKey::Has`.
    pub fn has(&self, info: &Information) -> bool {
        self.integer_key.has(info)
    }

    /// VTK: `vtkInformationKey::Remove`.
    pub fn remove(&self, info: &mut Information) {
        self.integer_key.remove(info);
    }

    /// VTK: `vtkInformationKey::GetName`.
    pub fn get_name(&self) -> Option<&str> {
        self.integer_key.get_name()
    }

    /// VTK: `vtkInformationKey::GetLocation`.
    pub fn get_location(&self) -> Option<&str> {
        self.integer_key.get_location()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.integer_key.get_class_name()
    }

    /// VTK: `vtkInformationIntegerRequestKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationIntegerRequestKey" || InformationIntegerKey::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerRequestKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationIntegerRequestKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationIntegerRequestKey" => 0,
            "vtkInformationIntegerKey" => 1,
            "vtkInformationKey" => 2,
            "vtkObjectBase" => 3,
            _ => InformationIntegerKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationIntegerRequestKey::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.integer_key.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.integer_key.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.integer_key.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.integer_key.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.integer_key.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.integer_key.set_reference_count(reference_count);
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.integer_key.get_object_description()
    }
}

impl InformationKeyRegistration for InformationIntegerRequestKey {
    fn information_key(&self) -> &InformationKey {
        self.integer_key.information_key()
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        self.integer_key.information_key_mut()
    }
}
