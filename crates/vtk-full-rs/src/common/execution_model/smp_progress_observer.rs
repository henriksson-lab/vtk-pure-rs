use std::{collections::HashMap, thread::ThreadId};

use crate::common::{core::VtkMTimeType, execution_model::ProgressObserver};

/// VTK: `vtkSMPProgressObserver`.
#[derive(Debug)]
pub struct SMPProgressObserver {
    progress_observer: ProgressObserver,
    observers: HashMap<ThreadId, ProgressObserver>,
}

impl SMPProgressObserver {
    /// VTK: `vtkSMPProgressObserver::New`.
    pub fn new() -> Self {
        Self {
            progress_observer: ProgressObserver::with_class_name("vtkSMPProgressObserver"),
            observers: HashMap::new(),
        }
    }

    /// VTK: `vtkSMPProgressObserver::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.progress_observer.print_self()
    }

    /// VTK: `vtkSMPProgressObserver::UpdateProgress`.
    pub fn update_progress(&mut self, progress: f64) {
        self.get_local_observer().update_progress(progress);
    }

    /// VTK: `vtkSMPProgressObserver::GetLocalObserver`.
    pub fn get_local_observer(&mut self) -> &mut ProgressObserver {
        let thread_id = std::thread::current().id();
        self.observers
            .entry(thread_id)
            .or_insert_with(ProgressObserver::new)
    }

    /// VTK: `vtkProgressObserver::GetProgress`.
    pub fn get_progress(&self) -> f64 {
        self.progress_observer.get_progress()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.progress_observer.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.progress_observer.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.progress_observer.get_class_name()
    }

    /// VTK: `vtkSMPProgressObserver::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkSMPProgressObserver" || ProgressObserver::is_type_of(name)
    }

    /// VTK: `vtkSMPProgressObserver::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.progress_observer.get_object_description()
    }
}

impl Default for SMPProgressObserver {
    fn default() -> Self {
        Self::new()
    }
}
