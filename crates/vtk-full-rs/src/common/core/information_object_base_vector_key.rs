use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, ObjectBaseHandle,
    VtkIdType,
};

#[derive(Debug)]
struct InformationObjectBaseVectorValue {
    base: ObjectBase,
    value: Vec<Option<ObjectBaseHandle>>,
}

impl InformationObjectBaseVectorValue {
    fn new() -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationObjectBaseVectorValue"),
            value: Vec::new(),
        }
    }

    fn print_value(&self) -> String {
        let mut output = String::new();
        for (index, value) in self.value.iter().enumerate() {
            output.push_str("item ");
            output.push_str(&index.to_string());
            output.push('=');
            match value {
                Some(value) => output.push_str(&value.print_self()),
                None => output.push_str("nullptr;"),
            }
            output.push('\n');
        }
        output
    }
}

impl InformationValue for InformationObjectBaseVectorValue {
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
        Box::new(Self {
            base: ObjectBase::with_class_name("vtkInformationObjectBaseVectorValue"),
            value: self.value.clone(),
        })
    }

    fn print_value(&self) -> String {
        InformationObjectBaseVectorValue::print_value(self)
    }
}

/// VTK: `vtkInformationObjectBaseVectorKey`.
#[derive(Debug)]
pub struct InformationObjectBaseVectorKey {
    information_key: InformationKey,
    required_class: Option<&'static str>,
}

impl InformationObjectBaseVectorKey {
    /// VTK: `vtkInformationObjectBaseVectorKey::vtkInformationObjectBaseVectorKey`.
    pub fn new(
        name: Option<&str>,
        location: Option<&str>,
        required_class: Option<&'static str>,
    ) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationObjectBaseVectorKey",
                name,
                location,
            ),
            required_class,
        })
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::MakeKey`.
    pub fn make_key(
        name: Option<&str>,
        location: Option<&str>,
        required_class: Option<&'static str>,
    ) -> *mut Self {
        Self::new(name, location, required_class)
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    fn validate_derived_type(&self, value: Option<&ObjectBaseHandle>) -> bool {
        match (value, self.required_class) {
            (Some(value), Some(required_class)) => value.is_a(required_class),
            _ => true,
        }
    }

    fn get_object_base_vector_mut<'a>(
        &self,
        info: &'a mut Information,
    ) -> &'a mut InformationObjectBaseVectorValue {
        let key = self.information_key.key_ptr();
        if info
            .get_as_object_base(key)
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationObjectBaseVectorValue>()
            })
            .is_none()
        {
            info.set_as_object_base(key, Some(Box::new(InformationObjectBaseVectorValue::new())));
        }
        info.get_as_object_base_mut(key)
            .and_then(|value| {
                value
                    .as_any_mut()
                    .downcast_mut::<InformationObjectBaseVectorValue>()
            })
            .expect("object-base vector value must exist after creation")
    }

    fn get_object_base_vector<'a>(
        &self,
        info: &'a Information,
    ) -> Option<&'a InformationObjectBaseVectorValue> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationObjectBaseVectorValue>()
            })
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Clear`.
    pub fn clear(&self, info: &mut Information) {
        self.get_object_base_vector_mut(info).value.clear();
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Resize`.
    pub fn resize(&self, info: &mut Information, size: i32) {
        let size = usize::try_from(size.max(0)).expect("non-negative size must fit usize");
        self.get_object_base_vector_mut(info)
            .value
            .resize(size, None);
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Size`.
    pub fn size(&self, info: &Information) -> i32 {
        self.get_object_base_vector(info)
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        self.size(info)
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: Option<ObjectBaseHandle>) {
        if !self.validate_derived_type(value.as_ref()) {
            return;
        }
        self.get_object_base_vector_mut(info).value.push(value);
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<ObjectBaseHandle>, index: i32) {
        if index < 0 || !self.validate_derived_type(value.as_ref()) {
            return;
        }
        let index = usize::try_from(index).expect("non-negative index must fit usize");
        let vector = &mut self.get_object_base_vector_mut(info).value;
        if index >= vector.len() {
            vector.resize(index + 1, None);
        }
        vector[index] = value;
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Remove(vtkObjectBase*)`.
    pub fn remove_value(&self, info: &mut Information, value: Option<&ObjectBaseHandle>) {
        if !self.validate_derived_type(value) {
            return;
        }
        let vector = &mut self.get_object_base_vector_mut(info).value;
        match value {
            Some(value) => vector.retain(|candidate| {
                candidate
                    .as_ref()
                    .is_none_or(|candidate| !candidate.ptr_eq(value))
            }),
            None => vector.retain(Option::is_some),
        }
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Remove(int)`.
    pub fn remove_index(&self, info: &mut Information, index: i32) {
        if index < 0 {
            return;
        }
        let index = usize::try_from(index).expect("non-negative index must fit usize");
        let vector = &mut self.get_object_base_vector_mut(info).value;
        if index < vector.len() {
            vector.remove(index);
        }
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::SetRange`.
    pub fn set_range(
        &self,
        info: &mut Information,
        source: &[Option<ObjectBaseHandle>],
        mut from: i32,
        mut to: i32,
        n: i32,
    ) {
        if from < 0 || to < 0 || n <= 0 {
            return;
        }
        let Some(required_i32) = to.checked_add(n) else {
            return;
        };
        let required = usize::try_from(required_i32).expect("range end must fit usize");
        let vector = &mut self.get_object_base_vector_mut(info).value;
        if required > vector.len() {
            vector.resize(required, None);
        }
        for _ in 0..n {
            let source_index = usize::try_from(from).expect("source index must fit usize");
            let dest_index = usize::try_from(to).expect("destination index must fit usize");
            if let Some(value) = source.get(source_index) {
                vector[dest_index] = value.clone();
            }
            from += 1;
            to += 1;
        }
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::GetRange`.
    pub fn get_range(
        &self,
        info: &Information,
        dest: &mut [Option<ObjectBaseHandle>],
        mut from: i32,
        mut to: i32,
        mut n: i32,
    ) {
        let Some(vector) = self.get_object_base_vector(info) else {
            return;
        };
        if from < 0 || to < 0 || n <= 0 {
            return;
        }
        let size = vector.value.len() as i32;
        if from >= size {
            return;
        }
        if n > size - from {
            n = size - from;
        }
        for _ in 0..n {
            let source_index = usize::try_from(from).expect("source index must fit usize");
            let dest_index = usize::try_from(to).expect("destination index must fit usize");
            if let Some(slot) = dest.get_mut(dest_index) {
                *slot = vector.value[source_index].clone();
            }
            from += 1;
            to += 1;
        }
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Get`.
    pub fn get(&self, info: &Information, index: i32) -> Option<ObjectBaseHandle> {
        if index < 0 {
            return None;
        }
        let index = usize::try_from(index).expect("non-negative index must fit usize");
        self.get_object_base_vector(info)
            .and_then(|vector| vector.value.get(index))
            .cloned()
            .flatten()
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, source: &Information, dest: &mut Information) {
        let key = self.information_key.key_ptr();
        let Some(source_vector) = self.get_object_base_vector(source) else {
            dest.remove(key);
            return;
        };
        let value = source_vector.value.clone();
        self.get_object_base_vector_mut(dest).value = value;
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.get_object_base_vector(info)
            .map_or_else(String::new, InformationObjectBaseVectorValue::print_value)
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

    /// VTK: `vtkInformationObjectBaseVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationObjectBaseVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationObjectBaseVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationObjectBaseVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationObjectBaseVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
