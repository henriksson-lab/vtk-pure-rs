use crate::common::{
    core::{
        information::InformationValue, information_key::InformationKeyRegistration,
        CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
    },
    data_model::DataObjectHandle,
};

#[derive(Debug)]
struct InformationDataObjectValue {
    base: ObjectBase,
    value: DataObjectHandle,
}

impl InformationDataObjectValue {
    fn new(value: DataObjectHandle) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkDataObject"),
            value,
        }
    }

    fn print_value(&self) -> String {
        let object = self.value.borrow();
        format!("{}({:p})", object.get_class_name(), &*object)
    }
}

impl InformationValue for InformationDataObjectValue {
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
        InformationDataObjectValue::print_value(self)
    }
}

/// VTK: `vtkInformationDataObjectKey`.
#[derive(Debug)]
pub struct InformationDataObjectKey {
    information_key: InformationKey,
}

impl InformationDataObjectKey {
    /// VTK: `vtkInformationDataObjectKey::vtkInformationDataObjectKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: Self::make_information_key(
                "vtkInformationDataObjectKey",
                name,
                location,
            ),
        })
    }

    pub(crate) fn with_class_name(
        class_name: &'static str,
        name: Option<&str>,
        location: Option<&str>,
    ) -> Self {
        Self {
            information_key: Self::make_information_key(class_name, name, location),
        }
    }

    fn make_information_key(
        class_name: &'static str,
        name: Option<&str>,
        location: Option<&str>,
    ) -> InformationKey {
        InformationKey::with_class_name(class_name, name, location)
    }

    /// VTK: `vtkInformationDataObjectKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationDataObjectKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationDataObjectKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<DataObjectHandle>) {
        let key = self.information_key.key_ptr();
        match value {
            Some(value) => {
                info.set_as_object_base(key, Some(Box::new(InformationDataObjectValue::new(value))))
            }
            None => info.remove(key),
        }
    }

    /// VTK: `vtkInformationDataObjectKey::Get`.
    pub fn get(&self, info: &Information) -> Option<DataObjectHandle> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationDataObjectValue>())
            .map(|value| value.value.clone())
    }

    /// VTK: `vtkInformationDataObjectKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationDataObjectKey::Report`.
    pub fn report(&self, _info: &Information) {}

    /// VTK: `vtkInformationKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationDataObjectValue>())
            .map_or_else(String::new, InformationDataObjectValue::print_value)
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

    /// VTK: `vtkInformationDataObjectKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationDataObjectKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationDataObjectKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationDataObjectKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationDataObjectKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationDataObjectKey::GetNumberOfGenerationsFromBase`.
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

    pub(crate) fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    pub(crate) fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}

impl InformationKeyRegistration for InformationDataObjectKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
