use crate::common::{
    core::{
        information::InformationValue, information_key::InformationKeyRegistration, Information,
        InformationKey, ObjectBase, VtkIdType,
    },
    execution_model::{ExecutiveHandle, FilteringInformationKeyManager},
};

#[derive(Debug)]
struct InformationExecutivePortVectorValue {
    base: ObjectBase,
    executives: Vec<ExecutiveHandle>,
    ports: Vec<i32>,
}

impl InformationExecutivePortVectorValue {
    fn new(executives: Vec<ExecutiveHandle>, ports: Vec<i32>) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationExecutivePortVectorValue"),
            executives,
            ports,
        }
    }

    fn print_value(&self) -> String {
        let mut output = String::new();
        let mut sep = "";
        for (executive, port) in self.executives.iter().zip(&self.ports) {
            output.push_str(sep);
            output.push_str(&format!(
                "{}({:p}) port {}",
                executive.get_class_name(),
                executive.as_ptr(),
                port
            ));
            sep = ", ";
        }
        output
    }
}

impl InformationValue for InformationExecutivePortVectorValue {
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
        Box::new(Self::new(self.executives.clone(), self.ports.clone()))
    }

    fn print_value(&self) -> String {
        InformationExecutivePortVectorValue::print_value(self)
    }
}

/// VTK: `vtkInformationExecutivePortVectorKey`.
#[derive(Debug)]
pub struct InformationExecutivePortVectorKey {
    information_key: InformationKey,
}

impl InformationExecutivePortVectorKey {
    /// VTK: `vtkInformationExecutivePortVectorKey::vtkInformationExecutivePortVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        FilteringInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationExecutivePortVectorKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    fn get_vector_mut<'a>(
        &self,
        info: &'a mut Information,
    ) -> Option<&'a mut InformationExecutivePortVectorValue> {
        info.get_as_object_base_mut(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any_mut()
                    .downcast_mut::<InformationExecutivePortVectorValue>()
            })
    }

    fn get_vector<'a>(
        &self,
        info: &'a Information,
    ) -> Option<&'a InformationExecutivePortVectorValue> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationExecutivePortVectorValue>()
            })
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Append`.
    pub fn append(&self, info: &mut Information, executive: ExecutiveHandle, port: i32) {
        if let Some(value) = self.get_vector_mut(info) {
            value.executives.push(executive);
            value.ports.push(port);
            info.modified();
            return;
        }
        self.set(info, &[executive], &[port]);
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Remove(vtkExecutive*, int)`.
    pub fn remove_value(&self, info: &mut Information, executive: &ExecutiveHandle, port: i32) {
        let key = self.information_key.key_ptr();
        let Some(value) = self.get_vector_mut(info) else {
            return;
        };
        if let Some(index) =
            value
                .executives
                .iter()
                .zip(&value.ports)
                .position(|(candidate, candidate_port)| {
                    candidate.ptr_eq(executive) && *candidate_port == port
                })
        {
            value.executives.remove(index);
            value.ports.remove(index);
            info.modified();
        }
        if self
            .get_vector(info)
            .is_some_and(|value| value.executives.is_empty())
        {
            info.remove(key);
        }
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Set`.
    pub fn set(&self, info: &mut Information, executives: &[ExecutiveHandle], ports: &[i32]) {
        let key = self.information_key.key_ptr();
        if executives.is_empty() || ports.is_empty() {
            info.remove(key);
            return;
        }
        let length = executives.len().min(ports.len());
        if length == 0 {
            info.remove(key);
            return;
        }

        let new_executives = executives[..length].to_vec();
        let new_ports = ports[..length].to_vec();
        if let Some(value) = self.get_vector_mut(info) {
            value.executives = new_executives;
            value.ports = new_ports;
            info.modified();
            return;
        }
        info.set_as_object_base(
            key,
            Some(Box::new(InformationExecutivePortVectorValue::new(
                new_executives,
                new_ports,
            ))),
        );
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::GetExecutives`.
    pub fn get_executives(&self, info: &Information) -> Vec<ExecutiveHandle> {
        self.get_vector(info)
            .map_or_else(Vec::new, |value| value.executives.clone())
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::GetPorts`.
    pub fn get_ports(&self, info: &Information) -> Vec<i32> {
        self.get_vector(info)
            .map_or_else(Vec::new, |value| value.ports.clone())
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Get`.
    pub fn get(&self, info: &Information) -> (Vec<ExecutiveHandle>, Vec<i32>) {
        (self.get_executives(info), self.get_ports(info))
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        self.get_vector(info)
            .map_or(0, |value| value.executives.len() as i32)
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        let key = self.information_key.key_ptr();
        let Some(value) = self.get_vector(from) else {
            to.remove(key);
            return;
        };
        self.set(to, &value.executives, &value.ports);
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::Report`.
    pub fn report(&self, _info: &Information) {}

    /// VTK: `vtkInformationKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.get_vector(info).map_or_else(
            String::new,
            InformationExecutivePortVectorValue::print_value,
        )
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

    /// VTK: `vtkInformationExecutivePortVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationExecutivePortVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationExecutivePortVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationExecutivePortVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationExecutivePortVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
