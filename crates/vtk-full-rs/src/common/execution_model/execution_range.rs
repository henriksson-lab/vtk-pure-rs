use crate::common::{
    core::{InformationVectorHandle, Object, VtkMTimeType},
    data_model::{DataObject, DataObjectHandle},
};

/// VTK: `vtkExecutionRange`.
#[derive(Debug)]
pub struct ExecutionRange {
    object: Object,
}

impl ExecutionRange {
    /// VTK: `vtkExecutionRange::vtkExecutionRange`.
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkExecutionRange")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
        }
    }

    /// VTK: `vtkExecutionRange::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_object_description()
    }

    /// VTK: `vtkExecutionRange::RequestDataObject`.
    pub fn request_data_object(
        &self,
        input_vector: Option<&[InformationVectorHandle]>,
        output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        let Some(input_vector) = input_vector else {
            return 0;
        };
        let Some(input_info_vector) = input_vector.first() else {
            return 0;
        };
        let Some(in_info) = input_info_vector.borrow().get_information_object(0) else {
            return 0;
        };
        let input = DataObject::data_object().get(&in_info.borrow());
        let Some(output_vector) = output_vector else {
            return 0;
        };
        let Some(out_info) = output_vector.borrow().get_information_object(0) else {
            return 0;
        };

        if let Some(input) = input {
            let output = input.borrow().new_instance();
            DataObject::data_object().set(
                &mut out_info.borrow_mut(),
                Some(DataObjectHandle::new(output)),
            );
        }
        1
    }

    /// VTK: `vtkExecutionRange::RequestInformation`.
    pub fn request_information(
        &self,
        _input_vector: Option<&[InformationVectorHandle]>,
        _output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        1
    }

    /// VTK: `vtkExecutionRange::RequestUpdateExtent`.
    pub fn request_update_extent(
        &self,
        _iteration: usize,
        _input_vector: Option<&[InformationVectorHandle]>,
        _output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        1
    }

    /// VTK: `vtkExecutionRange::RequestData`.
    pub fn request_data(
        &self,
        _iteration: usize,
        input_vector: Option<&[InformationVectorHandle]>,
        output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        let Some(input_vector) = input_vector else {
            return 0;
        };
        let Some(input_info_vector) = input_vector.first() else {
            return 0;
        };
        let Some(in_info) = input_info_vector.borrow().get_information_object(0) else {
            return 0;
        };
        let Some(input) = DataObject::data_object().get(&in_info.borrow()) else {
            return 0;
        };
        let Some(output_vector) = output_vector else {
            return 0;
        };
        let Some(out_info) = output_vector.borrow().get_information_object(0) else {
            return 0;
        };

        let mut output = input.borrow().new_instance();
        output.shallow_copy(&input.borrow());
        DataObject::data_object().set(
            &mut out_info.borrow_mut(),
            Some(DataObjectHandle::new(output)),
        );
        1
    }

    /// VTK: `vtkExecutionRange::Size`.
    pub fn size(&self) -> usize {
        1
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

    /// VTK: `vtkExecutionRange::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkExecutionRange" || Object::is_type_of(name)
    }

    /// VTK: `vtkExecutionRange::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}
