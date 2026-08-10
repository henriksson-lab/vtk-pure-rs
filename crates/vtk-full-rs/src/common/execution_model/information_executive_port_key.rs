use std::{cell::RefCell, fmt, rc::Rc};

use crate::common::{
    core::{
        information::InformationValue, information_key::InformationKeyRegistration, Information,
        InformationKey, ObjectBase, ObjectBaseApi, VtkIdType,
    },
    execution_model::FilteringInformationKeyManager,
};

/// Rust handle for APIs that store a `vtkExecutive*`.
pub trait ExecutiveApi: ObjectBaseApi {}

/// Shallow-copyable dynamic handle for `vtkExecutive*` storage.
#[derive(Clone)]
pub struct ExecutiveHandle {
    executive: Rc<RefCell<dyn ExecutiveApi>>,
}

impl ExecutiveHandle {
    pub fn new<T: ExecutiveApi + 'static>(executive: T) -> Self {
        Self {
            executive: Rc::new(RefCell::new(executive)),
        }
    }

    pub fn from_rc<T: ExecutiveApi + 'static>(executive: Rc<RefCell<T>>) -> Self {
        Self { executive }
    }

    pub fn as_ptr(&self) -> *const RefCell<dyn ExecutiveApi> {
        Rc::as_ptr(&self.executive)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.executive, &other.executive)
    }

    pub fn get_class_name(&self) -> String {
        self.executive.borrow().get_class_name().to_owned()
    }

    pub fn is_a(&self, name: &str) -> bool {
        self.executive.borrow().is_a(name)
    }

    pub fn get_object_description(&self) -> String {
        self.executive.borrow().get_object_description()
    }
}

impl fmt::Debug for ExecutiveHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutiveHandle")
            .field("class_name", &self.get_class_name())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct InformationExecutivePortValue {
    base: ObjectBase,
    executive: ExecutiveHandle,
    port: i32,
}

impl InformationExecutivePortValue {
    fn new(executive: ExecutiveHandle, port: i32) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationExecutivePortValue"),
            executive,
            port,
        }
    }

    fn print_value(&self) -> String {
        format!(
            "{}({:p}) port {}",
            self.executive.get_class_name(),
            self.executive.as_ptr(),
            self.port
        )
    }
}

impl InformationValue for InformationExecutivePortValue {
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
        Box::new(Self::new(self.executive.clone(), self.port))
    }

    fn print_value(&self) -> String {
        InformationExecutivePortValue::print_value(self)
    }
}

/// VTK: `vtkInformationExecutivePortKey`.
#[derive(Debug)]
pub struct InformationExecutivePortKey {
    information_key: InformationKey,
}

impl InformationExecutivePortKey {
    /// VTK: `vtkInformationExecutivePortKey::vtkInformationExecutivePortKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        FilteringInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationExecutivePortKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationExecutivePortKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationExecutivePortKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationExecutivePortKey::Set`.
    pub fn set(&self, info: &mut Information, executive: Option<ExecutiveHandle>, port: i32) {
        let key = self.information_key.key_ptr();
        let Some(executive) = executive else {
            info.remove(key);
            return;
        };

        if let Some(value) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationExecutivePortValue>()
        }) {
            value.executive = executive;
            value.port = port;
            info.modified();
            return;
        }

        info.set_as_object_base(
            key,
            Some(Box::new(InformationExecutivePortValue::new(
                executive, port,
            ))),
        );
    }

    /// VTK: `vtkInformationExecutivePortKey::GetExecutive`.
    pub fn get_executive(&self, info: &Information) -> Option<ExecutiveHandle> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationExecutivePortValue>()
            })
            .map(|value| value.executive.clone())
    }

    /// VTK: `vtkInformationExecutivePortKey::GetPort`.
    pub fn get_port(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationExecutivePortValue>()
            })
            .map_or(0, |value| value.port)
    }

    /// VTK: `vtkInformationExecutivePortKey::Get`.
    pub fn get(&self, info: &Information) -> (Option<ExecutiveHandle>, i32) {
        (self.get_executive(info), self.get_port(info))
    }

    /// VTK: `vtkInformationExecutivePortKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get_executive(from), self.get_port(from));
    }

    /// VTK: `vtkInformationExecutivePortKey::Report`.
    pub fn report(&self, _info: &Information) {}

    /// VTK: `vtkInformationKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationExecutivePortValue>()
            })
            .map_or_else(String::new, InformationExecutivePortValue::print_value)
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

    /// VTK: `vtkInformationExecutivePortKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationExecutivePortKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationExecutivePortKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationExecutivePortKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationExecutivePortKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationExecutivePortKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationExecutivePortKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
