use crate::common::core::{IdList, Points, VtkIdType, VtkMTimeType, VTK_DOUBLE, VTK_FLOAT};

use super::{CellIterator, CellIteratorApi, DataSetApi};

/// VTK: `vtkDataSetCellIterator`.
#[derive(Debug)]
pub struct DataSetCellIterator<'a> {
    cell_iterator: CellIterator,
    data_set: Option<*mut (dyn DataSetApi + 'a)>,
    cell_id: VtkIdType,
}

impl<'a> DataSetCellIterator<'a> {
    /// VTK: `vtkDataSetCellIterator::New`.
    pub fn new() -> Self {
        Self {
            cell_iterator: CellIterator::with_class_name("vtkDataSetCellIterator"),
            data_set: None,
            cell_id: 0,
        }
    }

    /// VTK protected friend method: `vtkDataSetCellIterator::SetDataSet`.
    pub(crate) fn set_data_set(&mut self, data_set: Option<&'a mut dyn DataSetApi>) {
        self.data_set = data_set.map(|data_set| data_set as *mut (dyn DataSetApi + 'a));
        self.cell_id = 0;

        if let Some(data_set) = self.data_set() {
            if data_set.get_class_name() == "vtkImageData" {
                self.cell_iterator.points_mut().set_data_type(VTK_DOUBLE);
            } else {
                set_array_type(
                    data_set.coordinate_data_types(),
                    self.cell_iterator.points_mut(),
                );
            }
        }
    }

    /// VTK: `vtkDataSetCellIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}DataSet: {:?}\n",
            self.cell_iterator.print_self(),
            self.data_set
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell_iterator.get_class_name()
    }

    /// VTK: `vtkDataSetCellIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkDataSetCellIterator" || CellIterator::is_type_of(name)
    }

    /// VTK: `vtkDataSetCellIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkDataSetCellIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkDataSetCellIterator" => 0,
            "vtkCellIterator" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => CellIterator::get_number_of_generations_from_base_type(name) + 1,
        }
    }

    /// VTK: `vtkDataSetCellIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.cell_iterator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.cell_iterator.get_m_time()
    }

    /// VTK: `vtkCellIterator::InitTraversal`.
    pub fn init_traversal(&mut self) {
        <Self as CellIteratorApi>::init_traversal(self);
    }

    /// VTK: `vtkCellIterator::GoToNextCell`.
    pub fn go_to_next_cell(&mut self) {
        <Self as CellIteratorApi>::go_to_next_cell(self);
    }

    /// VTK: `vtkDataSetCellIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> bool {
        <Self as CellIteratorApi>::is_done_with_traversal(self)
    }

    /// VTK: `vtkDataSetCellIterator::GetCellId`.
    pub fn get_cell_id(&self) -> VtkIdType {
        <Self as CellIteratorApi>::get_cell_id(self)
    }

    /// VTK: `vtkCellIterator::GetCellType`.
    pub fn get_cell_type(&mut self) -> i32 {
        <Self as CellIteratorApi>::get_cell_type(self)
    }

    /// VTK: `vtkCellIterator::GetCellDimension`.
    pub fn get_cell_dimension(&mut self) -> i32 {
        <Self as CellIteratorApi>::get_cell_dimension(self)
    }

    /// VTK: `vtkCellIterator::GetPointIds`.
    pub fn get_point_ids(&mut self) -> &IdList {
        <Self as CellIteratorApi>::get_point_ids(self)
    }

    /// VTK: `vtkCellIterator::GetPoints`.
    pub fn get_points(&mut self) -> &Points {
        <Self as CellIteratorApi>::get_points(self)
    }

    /// VTK: `vtkCellIterator::GetCellFaces`.
    pub fn get_cell_faces(&mut self) -> &super::CellArray {
        <Self as CellIteratorApi>::get_cell_faces(self)
    }

    /// VTK: `vtkCellIterator::GetSerializedCellFaces`.
    pub fn get_serialized_cell_faces(&mut self) -> &IdList {
        <Self as CellIteratorApi>::get_serialized_cell_faces(self)
    }

    /// VTK: `vtkCellIterator::GetNumberOfPoints`.
    pub fn get_number_of_points(&mut self) -> VtkIdType {
        <Self as CellIteratorApi>::get_number_of_points(self)
    }

    /// VTK: `vtkCellIterator::GetNumberOfFaces`.
    pub fn get_number_of_faces(&mut self) -> VtkIdType {
        <Self as CellIteratorApi>::get_number_of_faces(self)
    }

    fn data_set(&self) -> Option<&dyn DataSetApi> {
        self.data_set.map(|data_set| unsafe { &*data_set })
    }
}

impl CellIteratorApi for DataSetCellIterator<'_> {
    fn cell_iterator(&self) -> &CellIterator {
        &self.cell_iterator
    }

    fn cell_iterator_mut(&mut self) -> &mut CellIterator {
        &mut self.cell_iterator
    }

    fn is_done_with_traversal(&self) -> bool {
        self.data_set()
            .is_none_or(|data_set| self.cell_id >= data_set.get_number_of_cells())
    }

    fn get_cell_id(&self) -> VtkIdType {
        self.cell_id
    }

    fn reset_to_first_cell(&mut self) {
        self.cell_id = 0;
    }

    fn increment_to_next_cell(&mut self) {
        self.cell_id += 1;
    }

    fn fetch_cell_type(&mut self) {
        let cell_type = self
            .data_set()
            .expect("vtkDataSetCellIterator DataSet must be set")
            .get_cell_type(self.cell_id);
        self.cell_iterator.set_cell_type(cell_type);
    }

    fn fetch_point_ids(&mut self) {
        let mut point_ids = IdList::new();
        self.data_set()
            .expect("vtkDataSetCellIterator DataSet must be set")
            .get_cell_points(self.cell_id, &mut point_ids);
        self.cell_iterator.point_ids_mut().deep_copy(&point_ids);
    }

    fn fetch_points(&mut self) {
        let point_ids: Vec<_> = self.get_point_ids().iter().collect();
        let points: Vec<_> = {
            let data_set = self
                .data_set()
                .expect("vtkDataSetCellIterator DataSet must be set");
            point_ids
                .iter()
                .map(|&point_id| data_set.get_point(point_id))
                .collect()
        };

        let out_points = self.cell_iterator.points_mut();
        out_points.set_number_of_points(points.len() as VtkIdType);
        for (i, point) in points.into_iter().enumerate() {
            out_points.set_point(i as VtkIdType, point);
        }
    }
}

impl Default for DataSetCellIterator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn set_array_type(types: [Option<i32>; 3], points: &mut Points) {
    if types.contains(&Some(VTK_DOUBLE)) {
        points.set_data_type(VTK_DOUBLE);
        return;
    }

    let [x_type, y_type, z_type] = types;
    if x_type.is_some() || y_type.is_some() || z_type.is_some() {
        if x_type == y_type && x_type == z_type {
            points.set_data_type(x_type.expect("at least one coordinate type exists"));
            return;
        }

        if x_type.is_none() {
            if y_type.is_none() {
                points.set_data_type(z_type.expect("z coordinate type exists"));
                return;
            } else if z_type.is_none() || y_type == z_type {
                points.set_data_type(y_type.expect("y coordinate type exists"));
                return;
            }
        }

        if y_type.is_none() {
            if x_type.is_none() {
                points.set_data_type(z_type.expect("z coordinate type exists"));
                return;
            } else if z_type.is_none() || x_type == z_type {
                points.set_data_type(x_type.expect("x coordinate type exists"));
                return;
            }
        }

        if z_type.is_none() {
            if x_type.is_none() {
                points.set_data_type(y_type.expect("y coordinate type exists"));
                return;
            } else if y_type.is_none() || x_type == y_type {
                points.set_data_type(x_type.expect("x coordinate type exists"));
                return;
            }
        }
    }

    points.set_data_type(VTK_FLOAT);
}
