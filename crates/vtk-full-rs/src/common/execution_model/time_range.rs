use crate::common::{
    core::{InformationVectorHandle, VtkMTimeType},
    execution_model::{ExecutionRange, StreamingDemandDrivenPipeline},
};

/// VTK: `vtkTimeRange`.
#[derive(Debug)]
pub struct TimeRange {
    execution_range: ExecutionRange,
    number_of_time_steps: usize,
    time_values: Vec<f64>,
}

impl TimeRange {
    /// VTK: `vtkTimeRange::New`.
    pub fn new() -> Self {
        Self {
            execution_range: ExecutionRange::with_class_name("vtkTimeRange"),
            number_of_time_steps: 0,
            time_values: Vec::new(),
        }
    }

    /// VTK: `vtkTimeRange::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.execution_range.print_self();
        output.push_str("\nNumberOfTimeSteps: ");
        output.push_str(&self.number_of_time_steps.to_string());
        output.push_str("\nTimeValues: [");
        for value in &self.time_values {
            output.push(' ');
            output.push_str(&value.to_string());
        }
        output.push_str(" ]");
        output
    }

    /// VTK: `vtkTimeRange::RequestInformation`.
    pub fn request_information(
        &mut self,
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

        if let Some(time_steps) = StreamingDemandDrivenPipeline::time_steps().get(&in_info.borrow())
        {
            self.number_of_time_steps = time_steps.len();
            self.time_values.clear();
            self.time_values.extend_from_slice(time_steps);
        } else {
            self.number_of_time_steps = 1;
            self.time_values.clear();
            self.time_values.push(0.0);
        }

        let Some(output_vector) = output_vector else {
            return 0;
        };
        let Some(out_info) = output_vector.borrow().get_information_object(0) else {
            return 0;
        };
        let mut out_info = out_info.borrow_mut();
        StreamingDemandDrivenPipeline::time_steps().remove(&mut out_info);
        StreamingDemandDrivenPipeline::time_range().remove(&mut out_info);
        1
    }

    /// VTK: `vtkTimeRange::RequestUpdateExtent`.
    pub fn request_update_extent(
        &self,
        iteration: usize,
        input_vector: Option<&[InformationVectorHandle]>,
        _output_vector: Option<InformationVectorHandle>,
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

        if let Some(time_value) = self.time_values.get(iteration).copied() {
            StreamingDemandDrivenPipeline::update_time_step()
                .set(&mut in_info.borrow_mut(), time_value);
            1
        } else if self.time_values.is_empty() {
            1
        } else {
            0
        }
    }

    /// VTK: `vtkTimeRange::Size`.
    pub fn size(&self) -> usize {
        self.number_of_time_steps
    }

    /// VTK: `vtkExecutionRange::RequestDataObject`.
    pub fn request_data_object(
        &self,
        input_vector: Option<&[InformationVectorHandle]>,
        output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        self.execution_range
            .request_data_object(input_vector, output_vector)
    }

    /// VTK: `vtkExecutionRange::RequestData`.
    pub fn request_data(
        &self,
        iteration: usize,
        input_vector: Option<&[InformationVectorHandle]>,
        output_vector: Option<InformationVectorHandle>,
    ) -> i32 {
        self.execution_range
            .request_data(iteration, input_vector, output_vector)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.execution_range.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.execution_range.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.execution_range.get_class_name()
    }

    /// VTK: `vtkTimeRange::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkTimeRange" || ExecutionRange::is_type_of(name)
    }

    /// VTK: `vtkTimeRange::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.execution_range.get_object_description()
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        Self::new()
    }
}
