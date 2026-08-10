use crate::common::{
    core::{Object, VtkMTimeType},
    data_model::DataObjectHandle,
};

/// VTK: `vtkExecutionAggregator`.
#[derive(Debug)]
pub struct ExecutionAggregator {
    object: Object,
}

impl ExecutionAggregator {
    /// VTK: `vtkExecutionAggregator::vtkExecutionAggregator`.
    pub(crate) fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkExecutionAggregator"),
        }
    }

    /// VTK: `vtkExecutionAggregator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_object_description()
    }

    /// VTK: `vtkExecutionAggregator::RequestDataObject`.
    pub fn request_data_object(&self, input: Option<DataObjectHandle>) -> Option<DataObjectHandle> {
        input.map(|input| DataObjectHandle::new(input.borrow().new_instance()))
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

    /// VTK: `vtkExecutionAggregator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkExecutionAggregator" || Object::is_type_of(name)
    }

    /// VTK: `vtkExecutionAggregator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

/// VTK pure virtual surface for `vtkExecutionAggregator` subclasses.
pub trait ExecutionAggregatorApi {
    /// VTK: `vtkExecutionAggregator::Aggregate`.
    fn aggregate(&mut self, input: Option<DataObjectHandle>) -> bool;

    /// VTK: `vtkExecutionAggregator::GetOutputDataObject`.
    fn get_output_data_object(&self) -> Option<DataObjectHandle>;

    /// VTK: `vtkExecutionAggregator::Clear`.
    fn clear(&mut self);
}
