use crate::common::core::{IdList, VtkIdType};
use crate::common::data_model::{DataObjectType, DataSet, DataSetApi, ImageData, VTK_UNIFORM_GRID};

/// VTK: `vtkUniformGrid`.
#[derive(Debug, Clone, PartialEq)]
pub struct UniformGrid {
    image_data: ImageData,
}

impl UniformGrid {
    /// VTK: `vtkUniformGrid::New`.
    pub fn new() -> Self {
        Self {
            image_data: ImageData::with_type(DataObjectType::UniformGrid),
        }
    }

    /// VTK: `vtkUniformGrid::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        VTK_UNIFORM_GRID
    }

    /// VTK: `vtkImageData::Initialize`, exposed by `vtkUniformGrid`.
    pub fn initialize(&mut self) {
        self.image_data = ImageData::with_type(DataObjectType::UniformGrid);
    }

    /// VTK: `vtkUniformGrid::NewImageDataCopy`.
    pub fn new_image_data_copy(&self) -> ImageData {
        let mut copy = ImageData::new();
        copy.shallow_copy(&self.image_data);

        let extent = self.image_data.get_extent();
        let origin = self.image_data.get_origin();
        let spacing = self.image_data.get_spacing();
        copy.set_extent([0, -1, 0, -1, 0, -1]);
        copy.set_extent(extent);
        copy.set_origin(origin);
        copy.set_spacing(spacing);
        copy
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkUniformGrid"
    }
}

impl Default for UniformGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSetApi for UniformGrid {
    fn data_set(&self) -> &DataSet {
        self.image_data.data_set()
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        self.image_data.data_set_mut()
    }

    fn get_class_name(&self) -> &'static str {
        "vtkUniformGrid"
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
