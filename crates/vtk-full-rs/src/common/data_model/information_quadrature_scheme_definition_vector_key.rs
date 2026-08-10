use std::{cell::RefCell, rc::Rc};

use crate::common::{
    core::{
        information::InformationValue, information_key::InformationKeyRegistration,
        CommonInformationKeyManager, Information, InformationKey, ObjectBase, VtkIdType,
    },
    data_model::{
        QuadratureSchemeDefinition, QuadratureSchemeDefinitionHandle, XMLDataElementHandle,
        VTK_NUMBER_OF_CELL_TYPES,
    },
};

#[derive(Debug)]
struct InformationQuadratureSchemeDefinitionVectorValue {
    base: ObjectBase,
    value: Vec<Option<QuadratureSchemeDefinitionHandle>>,
}

impl InformationQuadratureSchemeDefinitionVectorValue {
    fn new() -> Self {
        Self {
            base: ObjectBase::with_class_name(
                "vtkInformationQuadratureSchemeDefinitionVectorValue",
            ),
            value: vec![None; VTK_NUMBER_OF_CELL_TYPES as usize],
        }
    }

    fn print_value(&self) -> String {
        let mut output = String::new();
        for (index, value) in self.value.iter().enumerate() {
            output.push_str("item ");
            output.push_str(&index.to_string());
            output.push('=');
            match value {
                Some(value) => output.push_str(&value.borrow().print_self()),
                None => output.push_str("nullptr;"),
            }
            output.push('\n');
        }
        output
    }
}

impl InformationValue for InformationQuadratureSchemeDefinitionVectorValue {
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
            let value = self
                .value
                .iter()
                .map(|definition| {
                    definition.as_ref().map(|definition| {
                        let source = definition.borrow();
                        let mut copy = QuadratureSchemeDefinition::new();
                        copy.deep_copy(&source);
                        Rc::new(RefCell::new(copy))
                    })
                })
                .collect();
            Box::new(Self {
                base: ObjectBase::with_class_name(
                    "vtkInformationQuadratureSchemeDefinitionVectorValue",
                ),
                value,
            })
        } else {
            Box::new(Self {
                base: ObjectBase::with_class_name(
                    "vtkInformationQuadratureSchemeDefinitionVectorValue",
                ),
                value: self.value.clone(),
            })
        }
    }

    fn print_value(&self) -> String {
        InformationQuadratureSchemeDefinitionVectorValue::print_value(self)
    }
}

/// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey`.
#[derive(Debug)]
pub struct InformationQuadratureSchemeDefinitionVectorKey {
    information_key: InformationKey,
}

impl InformationQuadratureSchemeDefinitionVectorKey {
    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::vtkInformationQuadratureSchemeDefinitionVectorKey`.
    pub fn new(name: Option<&str>, location: Option<&str>) -> *mut Self {
        CommonInformationKeyManager::register_owned(Self {
            information_key: InformationKey::with_class_name(
                "vtkInformationQuadratureSchemeDefinitionVectorKey",
                name,
                location,
            ),
        })
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.information_key.print_self()
    }

    fn get_quadrature_scheme_definition_vector_mut<'a>(
        &self,
        info: &'a mut Information,
    ) -> &'a mut InformationQuadratureSchemeDefinitionVectorValue {
        let key = self.information_key.key_ptr();
        if info
            .get_as_object_base(key)
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationQuadratureSchemeDefinitionVectorValue>()
            })
            .is_none()
        {
            info.set_as_object_base(
                key,
                Some(Box::new(
                    InformationQuadratureSchemeDefinitionVectorValue::new(),
                )),
            );
        }
        info.get_as_object_base_mut(key)
            .and_then(|value| {
                value
                    .as_any_mut()
                    .downcast_mut::<InformationQuadratureSchemeDefinitionVectorValue>()
            })
            .expect("quadrature scheme definition vector value must exist after creation")
    }

    fn get_quadrature_scheme_definition_vector<'a>(
        &self,
        info: &'a Information,
    ) -> Option<&'a InformationQuadratureSchemeDefinitionVectorValue> {
        info.get_as_object_base(self.information_key.key_ptr())
            .and_then(|value| {
                value
                    .as_any()
                    .downcast_ref::<InformationQuadratureSchemeDefinitionVectorValue>()
            })
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Append`.
    pub fn append(&self, info: &mut Information, value: Option<QuadratureSchemeDefinitionHandle>) {
        self.get_quadrature_scheme_definition_vector_mut(info)
            .value
            .push(value);
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Set`.
    pub fn set(
        &self,
        info: &mut Information,
        value: Option<QuadratureSchemeDefinitionHandle>,
        index: i32,
    ) {
        if index < 0 {
            return;
        }
        let index = usize::try_from(index).expect("non-negative index must fit usize");
        let vector = &mut self.get_quadrature_scheme_definition_vector_mut(info).value;
        if index >= vector.len() {
            vector.resize(index + 1, None);
        }
        vector[index] = value;
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::SetRange`.
    pub fn set_range(
        &self,
        info: &mut Information,
        source: &[Option<QuadratureSchemeDefinitionHandle>],
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
        let vector = &mut self.get_quadrature_scheme_definition_vector_mut(info).value;
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

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::GetRange`.
    pub fn get_range(
        &self,
        info: &Information,
        dest: &mut [Option<QuadratureSchemeDefinitionHandle>],
        mut from: i32,
        mut to: i32,
        mut n: i32,
    ) {
        let Some(vector) = self.get_quadrature_scheme_definition_vector(info) else {
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

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Get`.
    pub fn get(&self, info: &Information, index: i32) -> Option<QuadratureSchemeDefinitionHandle> {
        if index < 0 {
            return None;
        }
        let index = usize::try_from(index).expect("non-negative index must fit usize");
        self.get_quadrature_scheme_definition_vector(info)
            .and_then(|vector| vector.value.get(index))
            .cloned()
            .flatten()
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Size`.
    pub fn size(&self, info: &Information) -> i32 {
        self.get_quadrature_scheme_definition_vector(info)
            .map_or(0, |value| value.value.len() as i32)
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Length`.
    pub fn length(&self, info: &Information) -> i32 {
        self.size(info)
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Resize`.
    pub fn resize(&self, info: &mut Information, size: i32) {
        let size = usize::try_from(size.max(0)).expect("non-negative size must fit usize");
        self.get_quadrature_scheme_definition_vector_mut(info)
            .value
            .resize(size, None);
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Clear`.
    pub fn clear(&self, info: &mut Information) {
        self.get_quadrature_scheme_definition_vector_mut(info)
            .value
            .clear();
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::ShallowCopy`.
    pub fn shallow_copy(&self, source: &Information, dest: &mut Information) {
        let key = self.information_key.key_ptr();
        let Some(source_vector) = self.get_quadrature_scheme_definition_vector(source) else {
            dest.remove(key);
            return;
        };
        self.get_quadrature_scheme_definition_vector_mut(dest).value = source_vector.value.clone();
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::DeepCopy`.
    pub fn deep_copy(&self, source: &Information, dest: &mut Information) {
        let key = self.information_key.key_ptr();
        let Some(source_vector) = self.get_quadrature_scheme_definition_vector(source) else {
            dest.remove(key);
            return;
        };
        let dest_vector = &mut self.get_quadrature_scheme_definition_vector_mut(dest).value;
        dest_vector.clear();
        dest_vector.resize(source_vector.value.len(), None);
        for (index, source_definition) in source_vector.value.iter().enumerate() {
            if let Some(source_definition) = source_definition {
                let source_definition = source_definition.borrow();
                let mut dest_definition = QuadratureSchemeDefinition::new();
                dest_definition.deep_copy(&source_definition);
                dest_vector[index] = Some(Rc::new(RefCell::new(dest_definition)));
            }
        }
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::SaveState`.
    pub fn save_state(&self, info: &Information, root: &XMLDataElementHandle) -> i32 {
        let Some(source_vector) = self.get_quadrature_scheme_definition_vector(info) else {
            return 0;
        };
        if source_vector.value.is_empty() {
            return 0;
        }
        {
            let root_ref = root.borrow();
            if root_ref.get_name().is_some() || root_ref.get_number_of_nested_elements() > 0 {
                return 0;
            }
        }

        {
            let mut root_ref = root.borrow_mut();
            root_ref.set_name(Some("InformationKey"));
            root_ref.set_attribute(Some("name"), Some("DICTIONARY"));
            root_ref.set_attribute(Some("location"), Some("vtkQuadratureSchemeDefinition"));
        }

        for definition in source_vector.value.iter().flatten() {
            let element = crate::common::data_model::XMLDataElement::new();
            definition.borrow().save_state(&element);
            root.borrow_mut().add_nested_element(Some(element));
        }
        1
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::RestoreState`.
    pub fn restore_state(&self, info: &mut Information, root: &XMLDataElementHandle) -> i32 {
        let vector = &mut self.get_quadrature_scheme_definition_vector_mut(info).value;
        vector.clear();
        vector.resize(VTK_NUMBER_OF_CELL_TYPES as usize, None);

        {
            let root_ref = root.borrow();
            if root_ref.get_name() != Some("InformationKey")
                || root_ref.get_attribute(Some("name")) != Some("DICTIONARY")
                || root_ref.get_attribute(Some("location")) != Some("vtkQuadratureSchemeDefinition")
            {
                return 0;
            }
        }

        let number_of_definitions = root.borrow().get_number_of_nested_elements();
        for definition_id in 0..number_of_definitions {
            let Some(element) = root.borrow().get_nested_element(definition_id) else {
                continue;
            };
            let mut definition = QuadratureSchemeDefinition::new();
            if definition.restore_state(&element) != 0 {
                let cell_type = definition.get_cell_type();
                let Ok(index) = usize::try_from(cell_type) else {
                    continue;
                };
                if let Some(slot) = vector.get_mut(index) {
                    *slot = Some(Rc::new(RefCell::new(definition)));
                }
            }
        }
        1
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::Print`.
    pub fn print(&self, info: &Information) -> String {
        self.get_quadrature_scheme_definition_vector(info)
            .map_or_else(
                String::new,
                InformationQuadratureSchemeDefinitionVectorValue::print_value,
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

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationQuadratureSchemeDefinitionVectorKey"
            || InformationKey::is_type_of(name)
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationQuadratureSchemeDefinitionVectorKey" => 0,
            "vtkInformationKey" => 1,
            "vtkObjectBase" => 2,
            _ => InformationKey::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationQuadratureSchemeDefinitionVectorKey::GetNumberOfGenerationsFromBase`.
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

impl InformationKeyRegistration for InformationQuadratureSchemeDefinitionVectorKey {
    fn information_key(&self) -> &InformationKey {
        &self.information_key
    }

    fn information_key_mut(&mut self) -> &mut InformationKey {
        &mut self.information_key
    }
}
