use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkProgressObserver`.
#[derive(Debug, Clone)]
pub struct ProgressObserver {
    object: Object,
    progress: f64,
}

impl ProgressObserver {
    /// VTK: `vtkProgressObserver::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkProgressObserver")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            progress: 0.0,
        }
    }

    /// VTK: `vtkProgressObserver::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_object_description()
    }

    /// VTK: `vtkProgressObserver::UpdateProgress`.
    pub fn update_progress(&mut self, amount: f64) {
        self.progress = amount;
    }

    /// VTK: `vtkProgressObserver::GetProgress`.
    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkProgressObserver::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkProgressObserver" || Object::is_type_of(name)
    }

    /// VTK: `vtkProgressObserver::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for ProgressObserver {
    fn default() -> Self {
        Self::new()
    }
}
