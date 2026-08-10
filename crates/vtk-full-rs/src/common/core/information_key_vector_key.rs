use crate::common::core::{
    information::InformationValue, information_key::InformationKeyRegistration,
    CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
};

#[derive(Debug)]
struct InformationKeyVectorValue {
    base: ObjectBase,
    value: Vec<*mut InformationKey>,
}

impl InformationKeyVectorValue {
    fn new(value: &[*mut InformationKey]) -> Self {
        Self {
            base: ObjectBase::with_class_name("vtkInformationKeyVectorValue"),
            value: value.to_vec(),
        }
    }
}

impl InformationValue for InformationKeyVectorValue {
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
        Box::new(Self::new(&self.value))
    }

    fn print_value(&self) -> String {
        self.value
            .iter()
            .map(|key| {
                if key.is_null() {
                    "(nullptr)".to_owned()
                } else {
                    unsafe { &**key }.get_name().unwrap_or("").to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// VTK: `vtkInformationKeyVectorKey`.
#[derive(Debug)]
pub struct InformationKeyVectorKey {
    information_key: InformationKey,
}

impl InformationKeyVectorKey {
    /// VTK: `vtkInformationKeyVectorKey::vtkInformationKeyVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationKeyVectorKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationKeyVectorKey::MakeKey`.
    pub fn make_key(name: Option<&str>, location: Option<&str>) -> *mut Self {
        Self::new(name, location)
    }

    /// VTK: `vtkInformationKeyVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    /// VTK: `vtkInformationKeyVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: *mut InformationKey) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationKeyVectorValue>()
        }) {
            vector.value.push(value);
            info.modified_with_key(key);
        } else {
            self.set(info, Some(&[value]));
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::AppendUnique`.
    pub fn append_unique(&self, info: &mut Information, value: *mut InformationKey) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationKeyVectorValue>()
        }) {
            if !vector.value.contains(&value) {
                vector.value.push(value);
                info.modified_with_key(key);
            }
        } else {
            self.set(info, Some(&[value]));
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::Set`.
    pub fn set(&self, info: &mut Information, value: Option<&[*mut InformationKey]>) {
        let key = self.information_key.key_ptr();
        if let Some(value) = value {
            info.set_as_object_base(key, Some(Box::new(InformationKeyVectorValue::new(value))));
        } else {
            info.remove(key);
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::RemoveItem`.
    pub fn remove_item(&self, info: &mut Information, value: *mut InformationKey) {
        let key = self.information_key.key_ptr();
        if let Some(vector) = info.get_as_object_base_mut(key).and_then(|value| {
            value
                .as_any_mut()
                .downcast_mut::<InformationKeyVectorValue>()
        }) {
            if let Some(index) = vector.value.iter().position(|item| *item == value) {
                vector.value.remove(index);
                info.modified_with_key(key);
            }
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::Get()`.
    pub fn get<'a>(&self, info: &'a Information) -> Option<&'a [*mut InformationKey]> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationKeyVectorValue>())
            .and_then(|value| {
                if value.value.is_empty() {
                    None
                } else {
                    Some(value.value.as_slice())
                }
            })
    }

    /// VTK: `vtkInformationKeyVectorKey::Get(int)`.
    pub fn get_value(&self, info: &Information, index: i32) -> *mut InformationKey {
        if index < 0 || index >= self.length(info) {
            return std::ptr::null_mut();
        }
        self.get(info)
            .map_or(std::ptr::null_mut(), |value| value[index as usize])
    }

    /// VTK: `vtkInformationKeyVectorKey::Get(vtkInformationKey**)`.
    pub fn get_into(&self, info: &Information, value: &mut [*mut InformationKey]) {
        if let Some(source) = info
            .get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationKeyVectorValue>())
        {
            let count = value.len().min(source.value.len());
            value[..count].copy_from_slice(&source.value[..count]);
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| value.as_any().downcast_ref::<InformationKeyVectorValue>())
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationKeyVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, from: &Information, to: &mut Information) {
        self.set(to, self.get(from));
    }

    /// VTK: `vtkInformationKeyVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        if self.has(info) {
            info.get_as_object_base(self.information_key.key_ptr())
                .and_then(|value| value.as_any().downcast_ref::<InformationKeyVectorValue>())
                .map_or_else(String::new, InformationKeyVectorValue::print_value)
        } else {
            String::new()
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

    /// VTK: `vtkInformationKeyVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationKeyVectorKey" || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationKeyVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationKeyVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationKeyVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationKeyVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationKeyVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
