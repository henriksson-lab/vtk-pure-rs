use std::sync::atomic::{AtomicBool, Ordering};

use super::{object_base::ObjectBase, time_stamp::TimeStamp, vtk_type::VtkMTimeType};

static GLOBAL_WARNING_DISPLAY: AtomicBool = AtomicBool::new(true);

/// VTK: `vtkObject`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    base: ObjectBase,
    debug: bool,
    m_time: TimeStamp,
    object_name: String,
}

impl Object {
    /// VTK: `vtkObject::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkObject")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        let mut object = Self {
            base: ObjectBase::with_class_name(class_name),
            debug: false,
            m_time: TimeStamp::new(),
            object_name: String::new(),
        };
        object.modified();
        object
    }

    pub(crate) fn object_base_mut_ptr(&mut self) -> *mut ObjectBase {
        &mut self.base
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        GLOBAL_WARNING_DISPLAY.store(value, Ordering::Relaxed);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Self::set_global_warning_display(true);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Self::set_global_warning_display(false);
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        GLOBAL_WARNING_DISPLAY.load(Ordering::Relaxed)
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.debug = true;
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.debug = false;
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.debug
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {}

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.m_time.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.m_time.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.base.get_class_name()
    }

    /// VTK: `vtkObject::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkObject" || ObjectBase::is_type_of(name)
    }

    /// VTK: `vtkObject::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObject::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkObject" => 0,
            "vtkObjectBase" => 1,
            _ => ObjectBase::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkObject::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.base.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.base.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.base.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.base.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.base.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.base.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object_name = object_name.into();
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        &self.object_name
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        let description = self.base.get_object_description();
        if self.object_name.is_empty() {
            description
        } else {
            format!("{description} '{}'", self.object_name)
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}
