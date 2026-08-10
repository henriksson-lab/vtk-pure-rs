use crate::common::core::{IdList, VtkIdType};
use crate::common::data_model::{
    DataObjectType, DataSet, DataSetApi, ImageData, VTK_STRUCTURED_POINTS,
};

/// VTK: `vtkStructuredPoints`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredPoints {
    image_data: ImageData,
}

impl StructuredPoints {
    /// VTK: `vtkStructuredPoints::New`.
    pub fn new() -> Self {
        Self {
            image_data: ImageData::with_type(DataObjectType::StructuredPoints),
        }
    }

    /// VTK: `vtkStructuredPoints::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.image_data.print_self()
    }

    /// VTK: `vtkStructuredPoints::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        VTK_STRUCTURED_POINTS
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkStructuredPoints"
    }
}

impl Default for StructuredPoints {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSetApi for StructuredPoints {
    fn data_set(&self) -> &DataSet {
        self.image_data.data_set()
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        self.image_data.data_set_mut()
    }

    fn get_class_name(&self) -> &'static str {
        "vtkStructuredPoints"
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        self.image_data.get_number_of_cells()
    }

    fn get_number_of_points(&self) -> VtkIdType {
        self.image_data.get_number_of_points()
    }

    fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        self.image_data.get_cell_type(cell_id)
    }

    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        self.image_data.get_cell_points(cell_id, point_ids);
    }

    fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        self.image_data.get_point(point_id)
    }
}
