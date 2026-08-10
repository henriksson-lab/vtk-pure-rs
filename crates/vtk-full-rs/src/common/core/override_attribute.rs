use super::{
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkOverrideAttribute`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideAttribute {
    object: Object,
    name: String,
    value: String,
    next: Option<Box<OverrideAttribute>>,
}

impl OverrideAttribute {
    /// VTK: `vtkOverrideAttribute::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkOverrideAttribute"),
            name: String::new(),
            value: String::new(),
            next: None,
        }
    }

    /// VTK macro: `vtkGetCharFromStdStringMacro(Name)`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// VTK macro: `vtkGetCharFromStdStringMacro(Value)`.
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// VTK macro: `vtkGetSmartPointerMacro(Next, vtkOverrideAttribute)`.
    pub fn get_next(&self) -> Option<&OverrideAttribute> {
        self.next.as_deref()
    }

    /// VTK: `vtkOverrideAttribute::CreateAttributeChain`.
    pub fn create_attribute_chain(
        name: Option<&str>,
        value: Option<&str>,
        next_in_chain: Option<Self>,
    ) -> Self {
        Self {
            object: Object::with_class_name("vtkOverrideAttribute"),
            name: name.unwrap_or("").to_string(),
            value: value.unwrap_or("").to_string(),
            next: next_in_chain.map(Box::new),
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkOverrideAttribute::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkOverrideAttribute" || Object::is_type_of(name)
    }

    /// VTK: `vtkOverrideAttribute::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkOverrideAttribute::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkOverrideAttribute" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkOverrideAttribute::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        Object::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Object::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Object::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        Object::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.object.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.object.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.object.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.object.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        Object::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.object.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.object.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.object.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.object.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for OverrideAttribute {
    fn default() -> Self {
        Self::new()
    }
}
