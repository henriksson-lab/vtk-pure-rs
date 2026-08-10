use std::ffi::c_void;

use crate::common::core::{Object, OverrideAttribute, VtkMTimeType};

/// VTK: `vtkObjectFactory*`.
pub type ObjectFactoryHandle = *mut c_void;

/// VTK: `vtkOverrideInformation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideInformation {
    object: Object,
    class_override_name: Option<String>,
    class_override_with_name: Option<String>,
    description: Option<String>,
    object_factory: ObjectFactoryHandle,
    override_attributes: Option<Box<OverrideAttribute>>,
}

impl OverrideInformation {
    /// VTK: `vtkOverrideInformation::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkOverrideInformation"),
            class_override_name: None,
            class_override_with_name: None,
            description: None,
            object_factory: std::ptr::null_mut(),
            override_attributes: None,
        }
    }

    /// VTK: `vtkOverrideInformation::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = String::new();
        output.push_str("Override: ");
        match (
            self.class_override_name.as_deref(),
            self.class_override_with_name.as_deref(),
            self.description.as_deref(),
        ) {
            (Some(class_override_name), Some(class_override_with_name), Some(description)) => {
                output.push_str(class_override_name);
                output.push_str("\nWith: ");
                output.push_str(class_override_with_name);
                output.push_str("\nDescription: ");
                output.push_str(description);
            }
            _ => output.push_str("(none)\n"),
        }

        output.push_str("From Factory:\n");
        output.push_str(if self.object_factory.is_null() {
            "(none)\n"
        } else {
            "(set)\n"
        });
        output.push_str("Override Attributes:\n");
        output.push_str(
            self.override_attributes
                .as_ref()
                .map_or("(none)\n", |_| "(set)\n"),
        );
        output
    }

    /// VTK: `vtkOverrideInformation::GetClassOverrideName`.
    pub fn get_class_override_name(&self) -> Option<&str> {
        self.class_override_name.as_deref()
    }

    /// VTK: `vtkOverrideInformation::GetClassOverrideWithName`.
    pub fn get_class_override_with_name(&self) -> Option<&str> {
        self.class_override_with_name.as_deref()
    }

    /// VTK: `vtkOverrideInformation::GetDescription`.
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// VTK: `vtkOverrideInformation::GetObjectFactory`.
    pub fn get_object_factory(&self) -> ObjectFactoryHandle {
        self.object_factory
    }

    /// VTK: `vtkOverrideInformation::GetOverrideAttributes`.
    pub fn get_override_attributes(&self) -> Option<&OverrideAttribute> {
        self.override_attributes.as_deref()
    }

    /// VTK macro: `vtkSetStringMacro(ClassOverrideName)`.
    pub fn set_class_override_name(&mut self, class_override_name: Option<&str>) {
        self.class_override_name = class_override_name.map(str::to_string);
        self.modified();
    }

    /// VTK macro: `vtkSetStringMacro(ClassOverrideWithName)`.
    pub fn set_class_override_with_name(&mut self, class_override_with_name: Option<&str>) {
        self.class_override_with_name = class_override_with_name.map(str::to_string);
        self.modified();
    }

    /// VTK macro: `vtkSetStringMacro(Description)`.
    pub fn set_description(&mut self, description: Option<&str>) {
        self.description = description.map(str::to_string);
        self.modified();
    }

    /// VTK: `vtkOverrideInformation::SetObjectFactory`.
    #[allow(dead_code)]
    pub(crate) fn set_object_factory(&mut self, object_factory: ObjectFactoryHandle) {
        if self.object_factory != object_factory {
            self.object_factory = object_factory;
            self.modified();
        }
    }

    /// VTK: `vtkOverrideInformation::SetOverrideAttributes`.
    #[allow(dead_code)]
    pub(crate) fn set_override_attributes(
        &mut self,
        override_attributes: Option<OverrideAttribute>,
    ) {
        self.override_attributes = override_attributes.map(Box::new);
        self.modified();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for OverrideInformation {
    fn default() -> Self {
        Self::new()
    }
}
