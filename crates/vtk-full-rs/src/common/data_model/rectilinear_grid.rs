use super::{
    BoundingBox, CellType, DataObjectType, DataSet, DataSetApi, DataSetAttributes,
    DataSetAttributesFieldList, FieldData, StructuredCellArray, StructuredData, CELL, HIDDENCELL,
    HIDDENPOINT, POINT,
};
use crate::common::core::{
    AnyArray, DoubleArray, IdList, IntConstantArray, Points, StructuredPointArray,
    UnsignedCharConstantArray, VtkIdType,
};

/// Topologically regular grid with per-axis coordinate arrays.
///
/// VTK origin: `VTK/Common/DataModel/vtkRectilinearGrid.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct RectilinearGrid {
    x_coordinates: AnyArray,
    y_coordinates: AnyArray,
    z_coordinates: AnyArray,
    extent: [i32; 6],
    structured_points: Option<Points>,
    data_set: DataSet,
}

/// Implicit structured cell materialized from a rectilinear grid.
///
/// VTK origin: `vtkRectilinearGrid::GetCell`.
#[derive(Debug, Clone, PartialEq)]
pub struct RectilinearCell {
    pub cell_type: CellType,
    pub point_ids: Vec<VtkIdType>,
    pub points: Vec<[f64; 3]>,
}

impl RectilinearGrid {
    /// VTK: `vtkRectilinearGrid::New`.
    pub fn new() -> Self {
        Self {
            x_coordinates: coordinate_array("XCoordinates", vec![0.0]),
            y_coordinates: coordinate_array("YCoordinates", vec![0.0]),
            z_coordinates: coordinate_array("ZCoordinates", vec![0.0]),
            extent: [0, -1, 0, -1, 0, -1],
            structured_points: None,
            data_set: DataSet::with_type(DataObjectType::RectilinearGrid),
        }
    }

    /// Create a grid from x, y, and z coordinate arrays.
    ///
    /// VTK origin: `vtkRectilinearGrid::vtkRectilinearGrid` plus
    /// `SetXCoordinates`, `SetYCoordinates`, and `SetZCoordinates`.
    #[cfg(test)]
    fn from_coordinates(x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) -> Self {
        let extent = extent_from_dimensions([
            i32::try_from(x.len()).expect("x coordinate count must fit int"),
            i32::try_from(y.len()).expect("y coordinate count must fit int"),
            i32::try_from(z.len()).expect("z coordinate count must fit int"),
        ]);
        Self {
            x_coordinates: coordinate_array("XCoordinates", x),
            y_coordinates: coordinate_array("YCoordinates", y),
            z_coordinates: coordinate_array("ZCoordinates", z),
            extent,
            structured_points: None,
            data_set: DataSet::with_type(DataObjectType::RectilinearGrid),
        }
    }

    pub fn get_field_data(&self) -> &FieldData {
        self.data_set.data_object().get_field_data()
    }

    #[cfg(test)]
    pub(crate) fn get_field_data_mut(&mut self) -> &mut FieldData {
        self.data_set.data_object_mut().get_field_data_mut()
    }

    pub fn get_point_data(&self) -> &DataSetAttributes {
        self.data_set.get_point_data()
    }

    pub(crate) fn get_point_data_mut(&mut self) -> &mut DataSetAttributes {
        self.data_set.get_point_data_mut()
    }

    pub fn get_cell_data(&self) -> &DataSetAttributes {
        self.data_set.get_cell_data()
    }

    pub(crate) fn get_cell_data_mut(&mut self) -> &mut DataSetAttributes {
        self.data_set.get_cell_data_mut()
    }

    /// VTK: `vtkRectilinearGrid::Initialize`.
    ///
    /// VTK cleanup unregisters the three coordinate arrays and leaves null
    /// coordinate pointers; this reduced Rust form represents that as empty
    /// coordinate vectors and an empty extent.
    pub fn initialize(&mut self) {
        self.x_coordinates = coordinate_array("XCoordinates", Vec::new());
        self.y_coordinates = coordinate_array("YCoordinates", Vec::new());
        self.z_coordinates = coordinate_array("ZCoordinates", Vec::new());
        self.extent = [0, -1, 0, -1, 0, -1];
        self.structured_points = None;
        self.data_set.initialize();
    }

    /// VTK: `vtkRectilinearGrid::GetXCoordinates`.
    pub fn get_x_coordinates(&self) -> &AnyArray {
        &self.x_coordinates
    }

    /// VTK: `vtkRectilinearGrid::GetYCoordinates`.
    pub fn get_y_coordinates(&self) -> &AnyArray {
        &self.y_coordinates
    }

    /// VTK: `vtkRectilinearGrid::GetZCoordinates`.
    pub fn get_z_coordinates(&self) -> &AnyArray {
        &self.z_coordinates
    }

    fn x_coordinate_values(&self) -> Vec<f64> {
        coordinate_values(&self.x_coordinates)
    }

    fn y_coordinate_values(&self) -> Vec<f64> {
        coordinate_values(&self.y_coordinates)
    }

    fn z_coordinate_values(&self) -> Vec<f64> {
        coordinate_values(&self.z_coordinates)
    }

    /// VTK: `vtkRectilinearGrid::SetXCoordinates`.
    pub fn set_x_coordinates(&mut self, coordinates: AnyArray) {
        let mut next = self.x_coordinates.clone();
        set_coordinate_data(&mut next, coordinates, "XCoordinates");
        if self.x_coordinates == next {
            return;
        }
        self.x_coordinates = next;
        self.compute_bounds();
        self.build_points();
    }

    /// VTK: `vtkRectilinearGrid::SetYCoordinates`.
    pub fn set_y_coordinates(&mut self, coordinates: AnyArray) {
        let mut next = self.y_coordinates.clone();
        set_coordinate_data(&mut next, coordinates, "YCoordinates");
        if self.y_coordinates == next {
            return;
        }
        self.y_coordinates = next;
        self.compute_bounds();
        self.build_points();
    }

    /// VTK: `vtkRectilinearGrid::SetZCoordinates`.
    pub fn set_z_coordinates(&mut self, coordinates: AnyArray) {
        let mut next = self.z_coordinates.clone();
        set_coordinate_data(&mut next, coordinates, "ZCoordinates");
        if self.z_coordinates == next {
            return;
        }
        self.z_coordinates = next;
        self.compute_bounds();
        self.build_points();
    }

    #[cfg(test)]
    fn set_x_coordinate_values(&mut self, coordinates: Vec<f64>) {
        self.x_coordinates = coordinate_array("XCoordinates", coordinates)
    }

    #[cfg(test)]
    fn set_y_coordinate_values(&mut self, coordinates: Vec<f64>) {
        self.y_coordinates = coordinate_array("YCoordinates", coordinates)
    }

    #[cfg(test)]
    fn set_z_coordinate_values(&mut self, coordinates: Vec<f64>) {
        self.z_coordinates = coordinate_array("ZCoordinates", coordinates)
    }

    /// VTK: `vtkRectilinearGrid::GetDimensions`.
    pub fn get_dimensions(&self) -> [i32; 3] {
        StructuredData::get_dimensions_from_extent(self.extent)
    }

    /// VTK: `vtkRectilinearGrid::SetDimensions`.
    pub fn set_dimensions(&mut self, dimensions: [i32; 3]) {
        let extent = extent_from_dimensions(dimensions);
        if self.extent == extent {
            return;
        }
        self.extent = extent;
        self.resize_coordinates(dimensions);
        self.compute_bounds();
        self.build_points();
    }

    /// VTK: `vtkRectilinearGrid::GetExtent`.
    pub fn get_extent(&self) -> [i32; 6] {
        self.extent
    }

    /// VTK: `vtkRectilinearGrid::SetExtent`.
    pub fn set_extent(&mut self, extent: [i32; 6]) {
        if self.extent == extent {
            return;
        }
        self.extent = extent;
        self.resize_coordinates(StructuredData::get_dimensions_from_extent(extent));
        self.compute_bounds();
        self.build_points();
    }

    /// VTK: `vtkRectilinearGrid::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        StructuredData::get_number_of_points(self.extent)
    }

    /// VTK: `vtkRectilinearGrid::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        StructuredData::get_number_of_cells(self.extent)
    }

    /// VTK: inherited `vtkCartesianGrid::GetCells`.
    pub fn get_cells(&self) -> StructuredCellArray {
        StructuredData::get_cell_array(self.extent, true)
    }

    /// VTK: inherited `vtkCartesianGrid::GetCellTypes`.
    pub fn get_cell_types(&self) -> UnsignedCharConstantArray {
        StructuredData::get_cell_types(self.extent, true)
    }

    /// VTK: inherited `vtkCartesianGrid::GetCellTypesArray`.
    pub fn get_cell_types_array(&self) -> IntConstantArray {
        StructuredData::get_cell_types_array(self.extent, true)
    }

    /// VTK: inherited `vtkCartesianGrid::GetPoints`.
    pub fn get_points(&mut self) -> &Points {
        if self.structured_points.is_none() {
            self.build_points();
        }
        self.structured_points
            .as_ref()
            .expect("points cache exists")
    }

    /// VTK: `vtkDataSet::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            POINT => self.get_number_of_points(),
            CELL => self.get_number_of_cells(),
            _ => self.data_set.get_number_of_elements(attribute_type),
        }
    }

    /// VTK: `vtkRectilinearGrid::ComputePointId`.
    pub fn compute_point_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_point_id(self.get_dimensions(), ijk)
    }

    /// VTK: `vtkRectilinearGrid::ComputeCellId`.
    pub fn compute_cell_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_cell_id(self.get_dimensions(), ijk)
    }

    /// VTK: `vtkRectilinearGrid::GetPoint(vtkIdType, double*)`.
    pub fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        self.structured_points
            .as_ref()
            .map(|points| points.get_point(point_id))
            .unwrap_or_else(|| {
                let ijk = self
                    .ijk_from_point_id(point_id)
                    .expect("point id out of range");
                self.point_at_local_ijk(ijk)
                    .expect("coordinate arrays must contain the requested point")
            })
    }

    /// VTK: `vtkRectilinearGrid::GetPoint(int i, int j, int k, double*)`.
    pub fn get_point_at_ijk(&self, i: i32, j: i32, k: i32) -> [f64; 3] {
        let point_id = self.compute_point_id([i, j, k]);
        self.get_point(point_id)
    }

    /// VTK: `vtkRectilinearGrid::FindPoint`.
    pub fn find_point(&self, point: [f64; 3]) -> VtkIdType {
        let coordinates = [
            self.x_coordinate_values(),
            self.y_coordinate_values(),
            self.z_coordinate_values(),
        ];
        let mut loc = [0_i32; 3];

        for axis in 0..3 {
            let Some(axis_id) = nearest_axis_coordinate_id(&coordinates[axis], point[axis]) else {
                return -1;
            };
            loc[axis] = axis_id as i32;
        }

        self.compute_point_id(loc)
    }

    /// VTK: `vtkRectilinearGrid::ComputeStructuredCoordinates`.
    pub fn compute_structured_coordinates(
        &self,
        point: [f64; 3],
        ijk: &mut [i32; 3],
        pcoords: &mut [f64; 3],
    ) -> i32 {
        let dimensions = self.get_dimensions();
        let coordinates = [
            &self.x_coordinates,
            &self.y_coordinates,
            &self.z_coordinates,
        ];

        *ijk = [0; 3];
        *pcoords = [0.0; 3];

        for axis in 0..3 {
            let tuple_count = coordinates[axis].get_number_of_tuples();
            if tuple_count == 0 {
                return 0;
            }

            let mut previous = coordinate_value(coordinates[axis], 0)
                .expect("coordinate arrays must have one component");
            let next = coordinate_value(coordinates[axis], tuple_count as usize - 1)
                .expect("coordinate arrays must have one component");
            let (range_min, range_max) = if next < previous {
                (next, previous)
            } else {
                (previous, next)
            };
            if point[axis] < range_min || point[axis] > range_max {
                return 0;
            }
            if point[axis] == range_max && dimensions[axis] != 1 {
                return 0;
            }

            for tuple in 1..tuple_count as usize {
                let next = coordinate_value(coordinates[axis], tuple)
                    .expect("coordinate arrays must have one component");
                if point[axis] >= previous && point[axis] < next {
                    ijk[axis] = tuple as i32 - 1;
                    pcoords[axis] = (point[axis] - previous) / (next - previous);
                    break;
                } else if point[axis] == next {
                    ijk[axis] = tuple as i32 - 1;
                    pcoords[axis] = 1.0;
                    break;
                }
                previous = next;
            }
        }

        1
    }

    /// VTK: `vtkRectilinearGrid::FindCell`.
    pub fn find_cell(
        &self,
        point: [f64; 3],
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: Option<&mut [f64]>,
    ) -> VtkIdType {
        let mut loc = [0; 3];
        if self.compute_structured_coordinates(point, &mut loc, pcoords) == 0 {
            return -1;
        }

        if let Some(weights) = weights {
            voxel_interpolation_functions(*pcoords, weights);
        }

        *sub_id = 0;
        let cell_id = self.compute_cell_id(loc);
        if self.get_cell_type(cell_id) == CellType::Empty as i32 {
            return -1;
        }
        cell_id
    }

    /// VTK: `vtkRectilinearGrid::GetCell`.
    pub fn get_cell(&self, cell_id: VtkIdType) -> RectilinearCell {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            panic!("cell id out of range");
        }
        if !self.is_cell_visible(cell_id) {
            return RectilinearCell {
                cell_type: CellType::Empty,
                point_ids: Vec::new(),
                points: Vec::new(),
            };
        }
        let point_ids = StructuredData::cell_point_ids_for_extent(cell_id, self.extent, true);
        let points = point_ids
            .iter()
            .map(|&point_id| self.get_point(point_id))
            .collect();

        RectilinearCell {
            cell_type: self.cell_type(),
            point_ids,
            points,
        }
    }

    /// VTK: inherited `vtkCartesianGrid::GetCell(int, int, int)`.
    pub fn get_cell_ijk(&self, i: i32, j: i32, k: i32) -> RectilinearCell {
        self.get_cell(self.compute_cell_id([i, j, k]))
    }

    /// VTK: `vtkRectilinearGrid::GetCellType`.
    pub fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        if !self.is_cell_visible(cell_id) {
            return CellType::Empty as i32;
        }
        self.cell_type() as i32
    }

    /// VTK: inherited `vtkCartesianGrid::GetCellSize`.
    pub fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType {
        if !self.is_cell_visible(cell_id) {
            return 0;
        }
        StructuredData::cell_size_for_extent(cell_id, self.extent)
    }

    /// VTK: `vtkRectilinearGrid::GetCellPoints(vtkIdType, vtkIdList*)`.
    pub fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        point_ids.reset();
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        for point_id in self.get_cell(cell_id).point_ids {
            point_ids.insert_next_id(point_id);
        }
    }

    /// VTK: inherited `vtkCartesianGrid::GetCellNeighbors`.
    pub fn get_cell_neighbors(
        &self,
        cell_id: VtkIdType,
        point_ids: &IdList,
        cell_ids: &mut IdList,
    ) {
        match point_ids.get_number_of_ids() {
            0 => {
                cell_ids.reset();
                return;
            }
            1 | 2 | 4 => {
                StructuredData::get_cell_neighbors(
                    cell_id,
                    point_ids,
                    cell_ids,
                    self.get_dimensions(),
                );
            }
            _ => self.get_cell_neighbors_generic(cell_id, point_ids, cell_ids),
        }

        self.remove_invisible_cells(cell_ids);
    }

    /// VTK: inherited `vtkCartesianGrid::GetCellNeighbors` with seed location.
    pub fn get_cell_neighbors_with_seed(
        &self,
        cell_id: VtkIdType,
        point_ids: &IdList,
        cell_ids: &mut IdList,
        seed_loc: [i32; 3],
    ) {
        match point_ids.get_number_of_ids() {
            0 => {
                cell_ids.reset();
                return;
            }
            1 | 2 | 4 => {
                StructuredData::get_cell_neighbors_with_seed(
                    cell_id,
                    point_ids,
                    cell_ids,
                    self.get_dimensions(),
                    seed_loc,
                );
            }
            _ => self.get_cell_neighbors_generic(cell_id, point_ids, cell_ids),
        }

        self.remove_invisible_cells(cell_ids);
    }

    /// VTK: `vtkRectilinearGrid::GetCellBounds`.
    pub fn get_cell_bounds(&self, cell_id: VtkIdType) -> [f64; 6] {
        if StructuredData::cell_size_for_extent(cell_id, self.extent) == 0 {
            return [0.0; 6];
        }
        let mut bounds = BoundingBox::empty();
        for point_id in StructuredData::cell_point_ids_for_extent(cell_id, self.extent, true) {
            bounds.add_point(self.get_point(point_id));
        }
        bounds.get_bounds()
    }

    /// VTK: inherited `vtkCartesianGrid::GetPointCells`.
    pub fn get_point_cells(&self, point_id: VtkIdType, cell_ids: &mut IdList) {
        StructuredData::get_point_cells(point_id, cell_ids, self.get_dimensions());
    }

    /// VTK: inherited `vtkCartesianGrid::BlankPoint`.
    pub fn blank_point(&mut self, point_id: VtkIdType) {
        let number_of_points = self.get_number_of_points();
        if point_id < 0 || point_id >= number_of_points {
            return;
        }
        self.get_point_data_mut()
            .allocate_ghost_array(number_of_points);
        self.get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, true);
    }

    /// VTK: inherited `vtkCartesianGrid::BlankPoint(int, int, int)`.
    pub fn blank_point_ijk(&mut self, i: i32, j: i32, k: i32) {
        self.blank_point(self.compute_point_id([i, j, k]));
    }

    /// VTK: inherited `vtkCartesianGrid::UnBlankPoint`.
    pub fn un_blank_point(&mut self, point_id: VtkIdType) {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return;
        }
        self.get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, false);
    }

    /// VTK: inherited `vtkCartesianGrid::UnBlankPoint(int, int, int)`.
    pub fn un_blank_point_ijk(&mut self, i: i32, j: i32, k: i32) {
        self.un_blank_point(self.compute_point_id([i, j, k]));
    }

    /// VTK: inherited `vtkCartesianGrid::BlankCell`.
    pub fn blank_cell(&mut self, cell_id: VtkIdType) {
        let number_of_cells = self.get_number_of_cells();
        if cell_id < 0 || cell_id >= number_of_cells {
            return;
        }
        self.get_cell_data_mut()
            .allocate_ghost_array(number_of_cells);
        self.get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, true);
    }

    /// VTK: inherited `vtkCartesianGrid::BlankCell(int, int, int)`.
    pub fn blank_cell_ijk(&mut self, i: i32, j: i32, k: i32) {
        self.blank_cell(self.compute_cell_id([i, j, k]));
    }

    /// VTK: inherited `vtkCartesianGrid::UnBlankCell`.
    pub fn un_blank_cell(&mut self, cell_id: VtkIdType) {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        self.get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, false);
    }

    /// VTK: inherited `vtkCartesianGrid::UnBlankCell(int, int, int)`.
    pub fn un_blank_cell_ijk(&mut self, i: i32, j: i32, k: i32) {
        self.un_blank_cell(self.compute_cell_id([i, j, k]));
    }

    /// VTK: inherited `vtkCartesianGrid::IsPointVisible`.
    pub fn is_point_visible(&self, point_id: VtkIdType) -> bool {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return false;
        }
        StructuredData::is_point_visible(point_id, self.get_point_data().get_ghost_array())
    }

    /// VTK: inherited `vtkCartesianGrid::IsCellVisible`.
    pub fn is_cell_visible(&self, cell_id: VtkIdType) -> bool {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return false;
        }
        StructuredData::is_cell_visible(
            cell_id,
            self.get_dimensions(),
            StructuredData::get_data_description_from_extent(self.extent),
            self.get_cell_data().get_ghost_array(),
            self.get_point_data().get_ghost_array(),
        )
    }

    /// VTK: inherited `vtkCartesianGrid::HasAnyBlankPoints`.
    pub fn has_any_blank_points(&self) -> bool {
        self.get_point_data()
            .has_any_ghost_bit_set(i32::from(HIDDENPOINT))
    }

    /// VTK: inherited `vtkCartesianGrid::HasAnyBlankCells`.
    pub fn has_any_blank_cells(&self) -> bool {
        self.get_cell_data()
            .has_any_ghost_bit_set(i32::from(HIDDENCELL))
            || self.has_any_blank_points()
    }

    /// VTK: `vtkRectilinearGrid::ComputeBounds`.
    pub fn compute_bounds(&mut self) {
        if coordinate_len(&self.x_coordinates) == 0
            || coordinate_len(&self.y_coordinates) == 0
            || coordinate_len(&self.z_coordinates) == 0
        {
            self.data_set.set_bounds(BoundingBox::empty());
            return;
        }

        let x_first = coordinate_value(&self.x_coordinates, 0).unwrap();
        let y_first = coordinate_value(&self.y_coordinates, 0).unwrap();
        let z_first = coordinate_value(&self.z_coordinates, 0).unwrap();
        let x_last =
            coordinate_value(&self.x_coordinates, coordinate_len(&self.x_coordinates) - 1).unwrap();
        let y_last =
            coordinate_value(&self.y_coordinates, coordinate_len(&self.y_coordinates) - 1).unwrap();
        let z_last =
            coordinate_value(&self.z_coordinates, coordinate_len(&self.z_coordinates) - 1).unwrap();
        self.data_set.set_bounds(BoundingBox::from_bounds([
            x_first.min(x_last),
            x_first.max(x_last),
            y_first.min(y_last),
            y_first.max(y_last),
            z_first.min(z_last),
            z_first.max(z_last),
        ]));
    }

    /// VTK: `vtkRectilinearGrid::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.data_set.get_bounds()
    }

    /// VTK: `vtkRectilinearGrid::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.x_coordinates = other.x_coordinates.deep_clone();
        self.y_coordinates = other.y_coordinates.deep_clone();
        self.z_coordinates = other.z_coordinates.deep_clone();
        self.extent = other.extent;
        self.structured_points = other.structured_points.as_ref().map(|points| {
            let mut copy = Points::new();
            copy.deep_copy(points);
            copy
        });
        self.data_set.deep_copy(&other.data_set);
    }

    /// VTK: `vtkRectilinearGrid::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.x_coordinates = other.x_coordinates.shallow_clone();
        self.y_coordinates = other.y_coordinates.shallow_clone();
        self.z_coordinates = other.z_coordinates.shallow_clone();
        self.extent = other.extent;
        self.structured_points = other.structured_points.as_ref().map(|points| {
            let mut copy = Points::new();
            copy.shallow_copy(points);
            copy
        });
        self.data_set.shallow_copy(&other.data_set);
    }

    /// VTK: `vtkRectilinearGrid::Crop`.
    pub fn crop(&mut self, update_extent: [i32; 6]) {
        let extent = self.get_extent();
        if extent_is_empty(extent) {
            return;
        }

        let cropped_extent = intersect_extents(update_extent, extent);
        if cropped_extent == extent {
            return;
        }
        if extent_is_empty(cropped_extent) {
            return;
        }

        let x_coordinates = slice_coordinate_array(
            &self.x_coordinates,
            extent[0],
            cropped_extent[0],
            cropped_extent[1],
        );
        let y_coordinates = slice_coordinate_array(
            &self.y_coordinates,
            extent[2],
            cropped_extent[2],
            cropped_extent[3],
        );
        let z_coordinates = slice_coordinate_array(
            &self.z_coordinates,
            extent[4],
            cropped_extent[4],
            cropped_extent[5],
        );

        let source_point_data = self.get_point_data().shallow_clone();
        let source_cell_data = self.get_cell_data().shallow_clone();

        let mut new_point_data = DataSetAttributes::new();
        let mut point_fields = DataSetAttributesFieldList::new();
        point_fields.initialize_field_list(&source_point_data);
        new_point_data.copy_allocate(
            &mut point_fields,
            StructuredData::get_number_of_points(cropped_extent) as usize,
            0,
        );
        let _ =
            new_point_data.copy_structured_data(&source_point_data, extent, cropped_extent, true);

        let mut new_cell_data = DataSetAttributes::new();
        if let Some((input_cell_extent, output_cell_extent)) =
            crop_cell_extents(extent, cropped_extent)
        {
            let mut cell_fields = DataSetAttributesFieldList::new();
            cell_fields.initialize_field_list(&source_cell_data);
            new_cell_data.copy_allocate(
                &mut cell_fields,
                structured_extent_tuple_count(output_cell_extent),
                0,
            );
            let _ = new_cell_data.copy_structured_data(
                &source_cell_data,
                input_cell_extent,
                output_cell_extent,
                true,
            );
        }

        self.extent = cropped_extent;
        self.x_coordinates = x_coordinates;
        self.y_coordinates = y_coordinates;
        self.z_coordinates = z_coordinates;
        self.get_point_data_mut().shallow_copy(&new_point_data);
        self.get_cell_data_mut().shallow_copy(&new_cell_data);
        self.compute_bounds();
        self.build_points();
    }

    /// VTK: `vtkRectilinearGrid::CopyStructure`.
    pub fn copy_structure(&mut self, other: &Self) {
        self.initialize();
        self.extent = other.extent;
        self.x_coordinates = other.x_coordinates.shallow_clone();
        self.y_coordinates = other.y_coordinates.shallow_clone();
        self.z_coordinates = other.z_coordinates.shallow_clone();
        self.compute_bounds();
        self.build_points();

        if other.has_any_blank_points() {
            if let Some(ghosts) = other.get_point_data().get_ghost_array() {
                self.get_point_data_mut().add_array(ghosts.clone());
            }
        }
        if other.has_any_blank_cells() {
            if let Some(ghosts) = other.get_cell_data().get_ghost_array() {
                self.get_cell_data_mut().add_array(ghosts.clone());
            }
        }
    }

    /// VTK: `vtkRectilinearGrid::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.data_set.get_actual_memory_size()
            + self.x_coordinates.get_actual_memory_size()
            + self.y_coordinates.get_actual_memory_size()
            + self.z_coordinates.get_actual_memory_size()
    }

    /// VTK: `vtkRectilinearGrid::PrintSelf`.
    pub fn print_self(&self) -> String {
        let dimensions = self.get_dimensions();
        format!(
            "vtkRectilinearGrid\n  Dimensions: ({}, {}, {})\n  Extent: {}, {}, {}, {}, {}, {}\n  X Coordinates: {} values\n  Y Coordinates: {} values\n  Z Coordinates: {} values",
            dimensions[0],
            dimensions[1],
            dimensions[2],
            self.extent[0],
            self.extent[1],
            self.extent[2],
            self.extent[3],
            self.extent[4],
            self.extent[5],
            coordinate_len(&self.x_coordinates),
            coordinate_len(&self.y_coordinates),
            coordinate_len(&self.z_coordinates),
        )
    }

    fn resize_coordinates(&mut self, dimensions: [i32; 3]) {
        self.x_coordinates
            .set_number_of_tuples(VtkIdType::from(dimensions[0].max(0)));
        self.y_coordinates
            .set_number_of_tuples(VtkIdType::from(dimensions[1].max(0)));
        self.z_coordinates
            .set_number_of_tuples(VtkIdType::from(dimensions[2].max(0)));
    }

    /// VTK: `vtkRectilinearGrid::BuildPoints`.
    fn build_points(&mut self) {
        self.ensure_structured_points_storage();
        let extent = self.extent;
        let data_description = StructuredData::get_data_description_from_extent(extent);
        let number_of_points = StructuredData::get_number_of_points(extent);
        let points = self
            .structured_points
            .as_mut()
            .expect("structured points storage was created");
        let AnyArray::StructuredPoint(point_array) = points.get_data_mut() else {
            panic!("GetPoints()->GetData() is not a vtkStructuredPointArray");
        };
        point_array.construct_backend_without_direction_matrix(
            &self.x_coordinates,
            &self.y_coordinates,
            &self.z_coordinates,
            extent,
            data_description,
        );
        point_array.set_number_of_tuples(number_of_points);
    }

    fn ensure_structured_points_storage(&mut self) {
        if self.structured_points.is_some() {
            return;
        }
        let mut point_array = StructuredPointArray::new();
        point_array.set_number_of_components(3);
        let mut points = Points::new();
        points.set_data(&AnyArray::StructuredPoint(point_array));
        self.structured_points = Some(points);
    }

    fn point_at_local_ijk(&self, ijk: [usize; 3]) -> Option<[f64; 3]> {
        Some([
            coordinate_value(&self.x_coordinates, ijk[0])?,
            coordinate_value(&self.y_coordinates, ijk[1])?,
            coordinate_value(&self.z_coordinates, ijk[2])?,
        ])
    }

    fn ijk_from_point_id(&self, point_id: VtkIdType) -> Option<[usize; 3]> {
        let point_id = vtk_id_to_index(point_id)?;
        let dimensions = self.dimensions_as_sizes();
        if dimensions[0] == 0
            || dimensions[1] == 0
            || point_id >= self.get_number_of_points() as usize
        {
            return None;
        }
        let xy = dimensions[0] * dimensions[1];
        Some([
            point_id % dimensions[0],
            (point_id % xy) / dimensions[0],
            point_id / xy,
        ])
    }

    fn cell_type(&self) -> CellType {
        StructuredData::cell_type_for_extent(self.extent, true)
    }

    fn dimensions_as_sizes(&self) -> [usize; 3] {
        let dimensions = self.get_dimensions();
        [
            dimensions[0].max(0) as usize,
            dimensions[1].max(0) as usize,
            dimensions[2].max(0) as usize,
        ]
    }

    fn get_cell_neighbors_generic(
        &self,
        cell_id: VtkIdType,
        point_ids: &IdList,
        cell_ids: &mut IdList,
    ) {
        cell_ids.reset();
        if point_ids.get_number_of_ids() == 0 {
            return;
        }

        let query_point_ids: Vec<VtkIdType> = point_ids.iter().collect();
        for candidate_id in 0..self.get_number_of_cells() {
            if candidate_id == cell_id {
                continue;
            }
            let candidate_point_ids =
                StructuredData::cell_point_ids_for_extent(candidate_id, self.extent, true);
            if query_point_ids
                .iter()
                .all(|point_id| candidate_point_ids.contains(point_id))
            {
                cell_ids.insert_next_id(candidate_id);
            }
        }
    }

    fn remove_invisible_cells(&self, cell_ids: &mut IdList) {
        if !self.has_any_blank_cells() {
            return;
        }

        let visible_ids: Vec<VtkIdType> = cell_ids
            .iter()
            .filter(|cell_id| self.is_cell_visible(*cell_id))
            .collect();
        cell_ids.reset();
        for cell_id in visible_ids {
            cell_ids.insert_next_id(cell_id);
        }
        cell_ids.squeeze();
    }
}

impl DataSetApi for RectilinearGrid {
    fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        &mut self.data_set
    }

    fn get_class_name(&self) -> &'static str {
        "vtkRectilinearGrid"
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        RectilinearGrid::get_number_of_cells(self)
    }

    fn get_number_of_points(&self) -> VtkIdType {
        RectilinearGrid::get_number_of_points(self)
    }

    fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        RectilinearGrid::get_cell_type(self, cell_id)
    }

    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        RectilinearGrid::get_cell_points(self, cell_id, point_ids);
    }

    fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        RectilinearGrid::get_point(self, point_id)
    }

    fn coordinate_data_types(&self) -> [Option<i32>; 3] {
        [
            Some(self.x_coordinates.get_data_type().id()),
            Some(self.y_coordinates.get_data_type().id()),
            Some(self.z_coordinates.get_data_type().id()),
        ]
    }
}

fn vtk_id_to_index(id: VtkIdType) -> Option<usize> {
    usize::try_from(id).ok()
}

fn extent_from_dimensions(dimensions: [i32; 3]) -> [i32; 6] {
    if dimensions.contains(&0) {
        [0, -1, 0, -1, 0, -1]
    } else {
        [
            0,
            dimensions[0] - 1,
            0,
            dimensions[1] - 1,
            0,
            dimensions[2] - 1,
        ]
    }
}

fn nearest_axis_coordinate_id(coordinates: &[f64], value: f64) -> Option<usize> {
    let mut iter = coordinates.iter().copied();
    let mut previous = iter.next()?;
    let last = *coordinates.last()?;
    if value < previous || value > last {
        return None;
    }

    let mut loc = 0;
    for (i, next) in iter.enumerate() {
        let i = i + 1;
        if value >= previous && value <= next {
            loc = if (value - previous) < (next - value) {
                i - 1
            } else {
                i
            };
        }
        previous = next;
    }
    Some(loc)
}

fn coordinate_array(name: &str, coordinates: Vec<f64>) -> AnyArray {
    AnyArray::Double(DoubleArray::from_vec(name, coordinates, 1))
}

fn coordinate_values(coordinates: &AnyArray) -> Vec<f64> {
    (0..coordinates.get_number_of_tuples())
        .filter_map(|tuple| {
            coordinates
                .numeric_tuple_as_f64_checked(tuple as usize)
                .ok()
                .and_then(|values| values.first().copied())
        })
        .collect()
}

fn coordinate_value(coordinates: &AnyArray, tuple: usize) -> Option<f64> {
    coordinates
        .numeric_tuple_as_f64_checked(tuple)
        .ok()
        .and_then(|values| values.first().copied())
}

fn coordinate_len(coordinates: &AnyArray) -> usize {
    coordinates.get_number_of_tuples() as usize
}

fn extent_is_empty(extent: [i32; 6]) -> bool {
    extent[0] > extent[1] || extent[2] > extent[3] || extent[4] > extent[5]
}

fn intersect_extents(first: [i32; 6], second: [i32; 6]) -> [i32; 6] {
    [
        first[0].max(second[0]),
        first[1].min(second[1]),
        first[2].max(second[2]),
        first[3].min(second[3]),
        first[4].max(second[4]),
        first[5].min(second[5]),
    ]
}

fn crop_cell_extents(
    input_extent: [i32; 6],
    output_extent: [i32; 6],
) -> Option<([i32; 6], [i32; 6])> {
    let input_cell_extent = [
        input_extent[0],
        input_extent[1] - 1,
        input_extent[2],
        input_extent[3] - 1,
        input_extent[4],
        input_extent[5] - 1,
    ];
    let output_cell_extent = [
        output_extent[0],
        output_extent[1] - 1,
        output_extent[2],
        output_extent[3] - 1,
        output_extent[4],
        output_extent[5] - 1,
    ];
    if extent_is_empty(input_cell_extent) || extent_is_empty(output_cell_extent) {
        None
    } else {
        Some((input_cell_extent, output_cell_extent))
    }
}

fn structured_extent_tuple_count(extent: [i32; 6]) -> usize {
    if extent_is_empty(extent) {
        return 0;
    }
    (extent[1] - extent[0] + 1) as usize
        * (extent[3] - extent[2] + 1) as usize
        * (extent[5] - extent[4] + 1) as usize
}

fn slice_coordinate_array(
    coordinates: &AnyArray,
    input_min: i32,
    output_min: i32,
    output_max: i32,
) -> AnyArray {
    let mut sliced = coordinates.new_instance();
    let tuple_count = VtkIdType::from(output_max - output_min + 1);
    sliced.set_number_of_tuples(tuple_count);
    for idx in output_min..=output_max {
        let from_tuple = usize::try_from(idx - input_min).expect("cropped coordinate source index");
        let to_tuple = usize::try_from(idx - output_min).expect("cropped coordinate target index");
        let _ = sliced.copy_tuple_from(coordinates, from_tuple, to_tuple);
    }
    sliced
}

fn set_coordinate_data(target: &mut AnyArray, mut coordinates: AnyArray, name: &str) {
    if !coordinates.is_numeric() || coordinates.get_number_of_components() != 1 {
        return;
    }
    if coordinates.get_name().is_empty() {
        coordinates.set_name(name);
    }
    *target = coordinates;
}

fn voxel_interpolation_functions(pcoords: [f64; 3], weights: &mut [f64]) {
    let required = 8;
    if weights.len() < required {
        return;
    }

    let rm = 1.0 - pcoords[0];
    let sm = 1.0 - pcoords[1];
    let tm = 1.0 - pcoords[2];

    weights[0] = rm * sm * tm;
    weights[1] = pcoords[0] * sm * tm;
    weights[2] = rm * pcoords[1] * tm;
    weights[3] = pcoords[0] * pcoords[1] * tm;
    weights[4] = rm * sm * pcoords[2];
    weights[5] = pcoords[0] * sm * pcoords[2];
    weights[6] = rm * pcoords[1] * pcoords[2];
    weights[7] = pcoords[0] * pcoords[1] * pcoords[2];
}
