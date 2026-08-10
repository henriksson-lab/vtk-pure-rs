use crate::common::core::{IdList, IntConstantArray, Points, UnsignedCharConstantArray, VtkIdType};
use crate::common::data_model::{
    BoundingBox, CellType, DataObjectType, DataSet, DataSetApi, DataSetAttributes,
    DataSetAttributesFieldList, FieldDataArray, PointSet, StructuredCellArray, StructuredData,
    CELL, HIDDENCELL, HIDDENPOINT, POINT,
};

const VTK_DOUBLE_MIN: f64 = -1.0e299;
const VTK_DOUBLE_MAX: f64 = 1.0e299;

/// Curvilinear grid with explicit point coordinates.
///
/// VTK origin: `VTK/Common/DataModel/vtkStructuredGrid.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredGrid {
    dimensions: [i32; 3],
    extent: [i32; 6],
    point_set: PointSet,
}

/// Implicit structured cell materialized from a curvilinear grid.
///
/// VTK origin: `vtkStructuredGrid::GetCell`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredCell {
    pub cell_type: CellType,
    pub point_ids: Vec<VtkIdType>,
    pub points: Vec<[f64; 3]>,
}

/// VTK origin: `VTK_3D_EXTENT`.
pub const VTK_3D_EXTENT: i32 = 1;

impl StructuredGrid {
    /// VTK: `vtkStructuredGrid::New`.
    pub fn new() -> Self {
        Self {
            dimensions: [0, 0, 0],
            extent: [0, -1, 0, -1, 0, -1],
            point_set: PointSet::with_type(DataObjectType::StructuredGrid),
        }
    }

    /// VTK: `vtkStructuredGrid::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    /// VTK: `vtkStructuredGrid::GetDimensions`.
    pub fn get_dimensions(&self) -> [i32; 3] {
        self.dimensions
    }

    /// VTK: `vtkStructuredGrid::SetDimensions`.
    pub fn set_dimensions(&mut self, dimensions: [i32; 3]) {
        self.dimensions = dimensions;
        self.extent = extent_from_dimensions(dimensions);
    }

    /// VTK: `vtkStructuredGrid::SetExtent`.
    pub fn set_extent(&mut self, extent: [i32; 6]) {
        self.extent = extent;
        self.dimensions = StructuredData::get_dimensions_from_extent(extent);
    }

    /// VTK: `vtkStructuredGrid::GetExtent`.
    pub fn get_extent(&self) -> [i32; 6] {
        self.extent
    }

    /// VTK: `vtkStructuredGrid::GetPoints`.
    pub fn get_points(&self) -> Option<&Points> {
        self.point_set.get_points()
    }

    /// VTK: `vtkPointSet::SetPoints`.
    pub fn set_points(&mut self, points: Option<Points>) {
        self.point_set.set_points(points);
    }

    /// VTK: `vtkStructuredGrid::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.point_set.get_number_of_points()
    }

    /// VTK: `vtkStructuredGrid::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        StructuredData::get_number_of_cells(self.extent)
    }

    /// VTK: `vtkStructuredGrid::GetCells`.
    pub fn get_cells(&self) -> StructuredCellArray {
        StructuredData::get_cell_array(self.extent, false)
    }

    /// VTK: `vtkStructuredGrid::GetCellTypes`.
    pub fn get_cell_types(&self) -> UnsignedCharConstantArray {
        StructuredData::get_cell_types(self.extent, false)
    }

    /// VTK: `vtkStructuredGrid::GetCellTypesArray`.
    pub fn get_cell_types_array(&self) -> IntConstantArray {
        StructuredData::get_cell_types_array(self.extent, false)
    }

    /// VTK: `vtkDataSet::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            POINT => self.get_number_of_points(),
            CELL => self.get_number_of_cells(),
            _ => self.point_set.get_number_of_elements(attribute_type),
        }
    }

    /// VTK: `vtkStructuredData::ComputePointIdForExtent`.
    pub fn compute_point_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_point_id_for_extent(self.extent, ijk)
    }

    /// VTK: `vtkStructuredData::ComputeCellIdForExtent`.
    pub fn compute_cell_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_cell_id_for_extent(self.extent, ijk)
    }

    /// VTK: `vtkStructuredGrid::GetPoint`.
    pub fn get_point(&self, id: VtkIdType) -> [f64; 3] {
        self.point_set.get_point(id)
    }

    /// VTK: `vtkStructuredGrid::GetCell`.
    pub fn get_cell(&self, cell_id: VtkIdType) -> StructuredCell {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            panic!("cell id out of range");
        }
        if !self.is_cell_visible(cell_id) {
            return StructuredCell {
                cell_type: CellType::Empty,
                point_ids: Vec::new(),
                points: Vec::new(),
            };
        }
        let point_ids = StructuredData::cell_point_ids_for_extent(cell_id, self.extent, false);
        let points = point_ids
            .iter()
            .map(|&point_id| self.get_point(point_id))
            .collect();

        StructuredCell {
            cell_type: self.cell_type(),
            point_ids,
            points,
        }
    }

    /// VTK: `vtkStructuredGrid::GetCell(int, int, int)`.
    pub fn get_cell_ijk(&self, i: i32, j: i32, k: i32) -> StructuredCell {
        self.get_cell(self.compute_cell_id([i, j, k]))
    }

    /// VTK: `vtkStructuredGrid::GetCellType`.
    pub fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        if !self.is_cell_visible(cell_id) {
            return CellType::Empty as i32;
        }
        self.cell_type() as i32
    }

    /// VTK: `vtkStructuredGrid::GetCellSize`.
    pub fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType {
        if !self.is_cell_visible(cell_id) {
            return 0;
        }
        StructuredData::cell_size_for_extent(cell_id, self.extent)
    }

    /// VTK: `vtkStructuredGrid::GetCellPoints(vtkIdType, vtkIdList*)`.
    pub fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        point_ids.reset();
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        for point_id in StructuredData::cell_point_ids_for_extent(cell_id, self.extent, false) {
            point_ids.insert_next_id(point_id);
        }
    }

    /// VTK: `vtkStructuredGrid::GetCellNeighbors`.
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
                StructuredData::get_cell_neighbors(cell_id, point_ids, cell_ids, self.dimensions);
            }
            _ => self.get_cell_neighbors_generic(cell_id, point_ids, cell_ids),
        }

        self.remove_invisible_cells(cell_ids);
    }

    /// VTK: `vtkStructuredGrid::GetCellNeighbors` with seed location.
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
                    self.dimensions,
                    seed_loc,
                );
            }
            _ => self.get_cell_neighbors_generic(cell_id, point_ids, cell_ids),
        }

        self.remove_invisible_cells(cell_ids);
    }

    /// VTK: `vtkStructuredGrid::GetCellBounds`.
    pub fn get_cell_bounds(&self, cell_id: VtkIdType) -> [f64; 6] {
        if StructuredData::cell_size_for_extent(cell_id, self.extent) == 0 {
            return [0.0; 6];
        }
        let mut bounds = BoundingBox::empty();
        for point_id in StructuredData::cell_point_ids_for_extent(cell_id, self.extent, false) {
            bounds.add_point(self.get_point(point_id));
        }
        bounds.get_bounds()
    }

    /// VTK: `vtkStructuredGrid::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.point_set.get_bounds()
    }

    /// VTK: `vtkStructuredGrid::BlankPoint`.
    pub fn blank_point(&mut self, point_id: VtkIdType) {
        let number_of_points = self.get_number_of_points();
        if point_id < 0 || point_id >= number_of_points {
            return;
        }
        self.point_set
            .data_set_mut()
            .get_point_data_mut()
            .allocate_ghost_array(number_of_points);
        self.point_set
            .data_set_mut()
            .get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, true);
    }

    /// VTK: `vtkStructuredGrid::UnBlankPoint`.
    pub fn un_blank_point(&mut self, point_id: VtkIdType) {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return;
        }
        self.point_set
            .data_set_mut()
            .get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, false);
    }

    /// VTK: `vtkStructuredGrid::BlankCell`.
    pub fn blank_cell(&mut self, cell_id: VtkIdType) {
        let number_of_cells = self.get_number_of_cells();
        if cell_id < 0 || cell_id >= number_of_cells {
            return;
        }
        self.point_set
            .data_set_mut()
            .get_cell_data_mut()
            .allocate_ghost_array(number_of_cells);
        self.point_set
            .data_set_mut()
            .get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, true);
    }

    /// VTK: `vtkStructuredGrid::UnBlankCell`.
    pub fn un_blank_cell(&mut self, cell_id: VtkIdType) {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        self.point_set
            .data_set_mut()
            .get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, false);
    }

    /// VTK: `vtkStructuredGrid::IsPointVisible`.
    pub fn is_point_visible(&self, point_id: VtkIdType) -> bool {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return false;
        }
        StructuredData::is_point_visible(
            point_id,
            self.point_set.data_set().get_point_data().get_ghost_array(),
        )
    }

    /// VTK: `vtkStructuredGrid::IsCellVisible`.
    pub fn is_cell_visible(&self, cell_id: VtkIdType) -> bool {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return false;
        }
        StructuredData::is_cell_visible(
            cell_id,
            self.dimensions,
            StructuredData::get_data_description(self.dimensions),
            self.point_set.data_set().get_cell_data().get_ghost_array(),
            self.point_set.data_set().get_point_data().get_ghost_array(),
        )
    }

    /// VTK: `vtkStructuredGrid::HasAnyBlankPoints`.
    pub fn has_any_blank_points(&self) -> bool {
        self.point_set
            .data_set()
            .get_point_data()
            .has_any_ghost_bit_set(i32::from(HIDDENPOINT))
    }

    /// VTK: `vtkStructuredGrid::HasAnyBlankCells`.
    pub fn has_any_blank_cells(&self) -> bool {
        self.point_set
            .data_set()
            .get_cell_data()
            .has_any_ghost_bit_set(i32::from(HIDDENCELL))
            || self.has_any_blank_points()
    }

    /// VTK: `vtkStructuredGrid::ComputeScalarRange`.
    pub fn compute_scalar_range(&self) -> [f64; 2] {
        let mut point_range = [VTK_DOUBLE_MAX, VTK_DOUBLE_MIN];
        if let Some(point_scalars) = self
            .point_set
            .data_set()
            .get_point_data()
            .get_field_data_scalars()
        {
            self.accumulate_visible_scalar_range(
                point_scalars,
                self.get_number_of_points(),
                |grid, id| grid.is_point_visible(id),
                &mut point_range,
            );
        }

        let mut cell_range = point_range;
        if let Some(cell_scalars) = self
            .point_set
            .data_set()
            .get_cell_data()
            .get_field_data_scalars()
        {
            self.accumulate_visible_scalar_range(
                cell_scalars,
                self.get_number_of_cells(),
                |grid, id| grid.is_cell_visible(id),
                &mut cell_range,
            );
        }

        [
            if cell_range[0] >= VTK_DOUBLE_MAX {
                0.0
            } else {
                cell_range[0]
            },
            if cell_range[1] <= VTK_DOUBLE_MIN {
                1.0
            } else {
                cell_range[1]
            },
        ]
    }

    /// VTK: `vtkStructuredGrid::Crop`.
    pub fn crop(&mut self, update_extent: [i32; 6]) {
        let extent = self.extent;
        if structured_extent_is_empty(extent) {
            return;
        }

        let new_extent = intersect_extents(update_extent, extent);
        if new_extent == extent {
            return;
        }

        let Some(input_points) = self.get_points() else {
            return;
        };
        let input_points_data_type = input_points.get_data_type();

        if structured_extent_is_empty(new_extent) {
            self.set_extent(new_extent);
            self.set_points(Some(Points::new_with_data_type(input_points_data_type)));
            self.point_set
                .data_set_mut()
                .get_point_data_mut()
                .initialize();
            self.point_set
                .data_set_mut()
                .get_cell_data_mut()
                .initialize();
            return;
        }

        let output_size = StructuredData::get_number_of_points(new_extent);
        let mut output_points = Points::new_with_data_type(input_points_data_type);
        output_points.set_number_of_points(output_size);

        let source_point_data = self.point_set.data_set().get_point_data().shallow_clone();
        let source_cell_data = self.point_set.data_set().get_cell_data().shallow_clone();

        let mut output_point_data = DataSetAttributes::new();
        let mut point_fields = DataSetAttributesFieldList::new();
        point_fields.initialize_field_list(&source_point_data);
        output_point_data.copy_allocate(
            &mut point_fields,
            output_size as usize,
            output_size as usize,
        );

        let point_inc_i = VtkIdType::from(extent[1] - extent[0] + 1);
        let point_inc_j = point_inc_i * VtkIdType::from(extent[3] - extent[2] + 1);
        let mut output_id = 0_usize;
        for k in new_extent[4]..=new_extent[5] {
            let k_offset = VtkIdType::from(k - extent[4]) * point_inc_j;
            for j in new_extent[2]..=new_extent[3] {
                let j_offset = VtkIdType::from(j - extent[2]) * point_inc_i;
                for i in new_extent[0]..=new_extent[1] {
                    let input_id = VtkIdType::from(i - extent[0]) + j_offset + k_offset;
                    output_points
                        .set_point(output_id as VtkIdType, input_points.get_point(input_id));
                    let _ = output_point_data.copy_data(
                        &point_fields,
                        0,
                        &source_point_data,
                        input_id as usize,
                        output_id,
                    );
                    output_id += 1;
                }
            }
        }

        let mut output_cell_data = DataSetAttributes::new();
        let mut cell_fields = DataSetAttributesFieldList::new();
        cell_fields.initialize_field_list(&source_cell_data);
        output_cell_data.copy_allocate(
            &mut cell_fields,
            output_size as usize,
            output_size as usize,
        );

        let cell_inc_i = VtkIdType::from(extent[1] - extent[0]);
        let cell_inc_j = cell_inc_i * VtkIdType::from(extent[3] - extent[2]);
        output_id = 0;
        for k in new_extent[4]..new_extent[5] {
            let k_offset = VtkIdType::from(k - extent[4]) * cell_inc_j;
            for j in new_extent[2]..new_extent[3] {
                let j_offset = VtkIdType::from(j - extent[2]) * cell_inc_i;
                for i in new_extent[0]..new_extent[1] {
                    let input_id = VtkIdType::from(i - extent[0]) + j_offset + k_offset;
                    let _ = output_cell_data.copy_data(
                        &cell_fields,
                        0,
                        &source_cell_data,
                        input_id as usize,
                        output_id,
                    );
                    output_id += 1;
                }
            }
        }

        self.set_extent(new_extent);
        self.set_points(Some(output_points));
        self.point_set
            .data_set_mut()
            .get_point_data_mut()
            .shallow_copy(&output_point_data);
        self.point_set
            .data_set_mut()
            .get_cell_data_mut()
            .shallow_copy(&output_cell_data);
    }

    fn accumulate_visible_scalar_range(
        &self,
        scalars: &FieldDataArray,
        number_of_tuples: VtkIdType,
        is_visible: impl Fn(&Self, VtkIdType) -> bool,
        range: &mut [f64; 2],
    ) {
        for tuple_id in 0..number_of_tuples {
            if !is_visible(self, tuple_id) {
                continue;
            }
            let Ok(tuple) = scalars
                .get_data()
                .numeric_tuple_as_f64_checked(tuple_id as usize)
            else {
                continue;
            };
            let Some(value) = tuple.first().copied() else {
                continue;
            };
            range[0] = if range[0] < value { range[0] } else { value };
            range[1] = if value < range[1] { range[1] } else { value };
        }
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

        let query_point_ids: Vec<_> = point_ids.iter().collect();
        for candidate_id in 0..self.get_number_of_cells() {
            if candidate_id == cell_id {
                continue;
            }
            let candidate_point_ids =
                StructuredData::cell_point_ids_for_extent(candidate_id, self.extent, false);
            if query_point_ids
                .iter()
                .all(|point_id| candidate_point_ids.contains(point_id))
            {
                cell_ids.insert_next_id(candidate_id);
            }
        }
    }

    fn remove_invisible_cells(&self, cell_ids: &mut IdList) {
        if self
            .point_set
            .data_set()
            .get_point_data()
            .get_ghost_array()
            .is_none()
            && self
                .point_set
                .data_set()
                .get_cell_data()
                .get_ghost_array()
                .is_none()
        {
            return;
        }

        let visible_cell_ids: Vec<_> = cell_ids
            .iter()
            .filter(|cell_id| self.is_cell_visible(*cell_id))
            .collect();
        cell_ids.reset();
        for cell_id in visible_cell_ids {
            cell_ids.insert_next_id(cell_id);
        }
        cell_ids.squeeze();
    }

    /// VTK: `vtkStructuredGrid::CopyStructure`.
    pub fn copy_structure(&mut self, source: &Self) {
        self.point_set.copy_structure(&source.point_set);
        self.set_extent(source.extent);

        if source.has_any_blank_points() {
            if let Some(ghosts) = source
                .point_set
                .data_set()
                .get_point_data()
                .get_ghost_array()
                .cloned()
            {
                self.point_set
                    .data_set_mut()
                    .get_point_data_mut()
                    .add_array(ghosts);
            }
        }

        if source.has_any_blank_cells() {
            if let Some(ghosts) = source
                .point_set
                .data_set()
                .get_cell_data()
                .get_ghost_array()
                .cloned()
            {
                self.point_set
                    .data_set_mut()
                    .get_cell_data_mut()
                    .add_array(ghosts);
            }
        }
    }

    /// VTK: `vtkStructuredGrid::Initialize`.
    pub fn initialize(&mut self) {
        *self = Self::new();
    }

    /// VTK: `vtkStructuredGrid::GetCellDims`.
    pub fn get_cell_dims(&self) -> [i32; 3] {
        StructuredData::get_cell_dimensions_from_point_dimensions(self.dimensions)
            .map(|value| value.max(1))
    }

    /// VTK: `vtkStructuredGrid::GetDataDimension`.
    pub fn get_data_dimension(&self) -> i32 {
        StructuredData::get_data_dimension_from_extent(self.extent)
    }

    /// VTK: `vtkStructuredGrid::GetMaxSpatialDimension`.
    pub fn get_max_spatial_dimension(&self) -> i32 {
        self.get_data_dimension()
    }

    /// VTK: `vtkStructuredGrid::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        8
    }

    /// VTK: `vtkStructuredGrid::GetMinSpatialDimension`.
    pub fn get_min_spatial_dimension(&self) -> i32 {
        self.get_data_dimension()
    }

    /// VTK: `vtkStructuredGrid::GetExtentType`.
    pub fn get_extent_type(&self) -> i32 {
        VTK_3D_EXTENT
    }

    /// VTK: `vtkStructuredGrid::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.point_set.get_actual_memory_size()
    }

    /// VTK: `vtkStructuredGrid::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.dimensions = other.dimensions;
        self.extent = other.extent;
        self.point_set.deep_copy(&other.point_set);
    }

    /// VTK: `vtkStructuredGrid::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.dimensions = other.dimensions;
        self.extent = other.extent;
        self.point_set.shallow_copy(&other.point_set);
    }

    /// VTK: `vtkStructuredGrid::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkStructuredGrid\n  Dimensions: ({}, {}, {})\n  Extent: {}, {}, {}, {}, {}, {}",
            self.dimensions[0],
            self.dimensions[1],
            self.dimensions[2],
            self.extent[0],
            self.extent[1],
            self.extent[2],
            self.extent[3],
            self.extent[4],
            self.extent[5]
        )
    }

    fn cell_type(&self) -> CellType {
        StructuredData::cell_type_for_extent(self.extent, false)
    }
}

impl DataSetApi for StructuredGrid {
    fn data_set(&self) -> &DataSet {
        self.point_set.data_set()
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        self.point_set.data_set_mut()
    }

    fn get_class_name(&self) -> &'static str {
        "vtkStructuredGrid"
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        StructuredGrid::get_number_of_cells(self)
    }

    fn get_number_of_points(&self) -> VtkIdType {
        StructuredGrid::get_number_of_points(self)
    }

    fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        StructuredGrid::get_cell_type(self, cell_id)
    }

    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        StructuredGrid::get_cell_points(self, cell_id, point_ids);
    }

    fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        StructuredGrid::get_point(self, point_id)
    }
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

fn structured_extent_is_empty(extent: [i32; 6]) -> bool {
    extent[0] > extent[1] || extent[2] > extent[3] || extent[4] > extent[5]
}

fn intersect_extents(lhs: [i32; 6], rhs: [i32; 6]) -> [i32; 6] {
    [
        lhs[0].max(rhs[0]),
        lhs[1].min(rhs[1]),
        lhs[2].max(rhs[2]),
        lhs[3].min(rhs[3]),
        lhs[4].max(rhs[4]),
        lhs[5].min(rhs[5]),
    ]
}
