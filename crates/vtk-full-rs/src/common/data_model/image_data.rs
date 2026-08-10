use super::{
    BoundingBox, CellType, DataObjectType, DataSet, DataSetApi, DataSetAttributes,
    DataSetAttributesFieldList, FieldData, FieldDataArray, StructuredCellArray, StructuredData,
    CELL, HIDDENCELL, HIDDENPOINT, POINT, VTK_STRUCTURED_XY_PLANE, VTK_STRUCTURED_XZ_PLANE,
    VTK_STRUCTURED_YZ_PLANE,
};
use crate::common::core::{
    AnyArray, DoubleArray, IdList, IntConstantArray, Points, StructuredPointArray,
    UnsignedCharConstantArray, VtkDataType, VtkIdType,
};

const VTK_DOUBLE_MIN: f64 = -1.0e299;
const VTK_DOUBLE_MAX: f64 = 1.0e299;

/// Regular image grid with implicit point coordinates.
///
/// VTK origin: `VTK/Common/DataModel/vtkImageData.cxx`.
///
/// The extent is `[x_min, x_max, y_min, y_max, z_min, z_max]` in point index
/// space. The physical coordinate for index `(i, j, k)` is
/// `origin + spacing * [i, j, k]`; like VTK, `origin` is the coordinate of
/// global index `(0, 0, 0)` even when the extent starts elsewhere.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageData {
    extent: [i32; 6],
    origin: [f64; 3],
    spacing: [f64; 3],
    direction_matrix: [f64; 9],
    structured_points: Option<Points>,
    data_set: DataSet,
}

/// Implicit image cell materialized from `vtkImageData`.
///
/// VTK origin: `vtkImageData::GetCell`.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCell {
    pub cell_type: CellType,
    pub point_ids: Vec<VtkIdType>,
    pub points: Vec<[f64; 3]>,
}

impl ImageData {
    /// VTK: `vtkImageData::New`.
    pub fn new() -> Self {
        Self::with_type(DataObjectType::ImageData)
    }

    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            extent: [0, -1, 0, -1, 0, -1],
            origin: [0.0; 3],
            spacing: [1.0; 3],
            direction_matrix: identity_direction(),
            structured_points: None,
            data_set: DataSet::with_type(object_type),
        }
    }

    /// VTK: `vtkImageData::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    /// VTK: `vtkImageData::Initialize`.
    pub fn initialize(&mut self) {
        *self = Self::new();
    }

    /// VTK: `vtkImageData::CopyStructure`.
    pub fn copy_structure(&mut self, source: &ImageData) {
        self.initialize();
        self.spacing = source.spacing;
        self.origin = source.origin;
        self.direction_matrix = source.direction_matrix;
        self.set_extent(source.extent);

        if source.has_any_blank_points() {
            if let Some(ghosts) = source.get_point_data().get_ghost_array() {
                self.get_point_data_mut().add_array(ghosts.clone());
            }
        }
        if source.has_any_blank_cells() {
            if let Some(ghosts) = source.get_cell_data().get_ghost_array() {
                self.get_cell_data_mut().add_array(ghosts.clone());
            }
        }
    }

    /// VTK: `vtkImageData::DeepCopy`.
    pub fn deep_copy(&mut self, source: &ImageData) {
        self.extent = source.extent;
        self.origin = source.origin;
        self.spacing = source.spacing;
        self.direction_matrix = source.direction_matrix;
        self.structured_points = source.structured_points.as_ref().map(|points| {
            let mut copy = Points::new();
            copy.deep_copy(points);
            copy
        });
        self.data_set.deep_copy(&source.data_set);
    }

    /// VTK: `vtkImageData::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &ImageData) {
        self.extent = source.extent;
        self.origin = source.origin;
        self.spacing = source.spacing;
        self.direction_matrix = source.direction_matrix;
        self.structured_points = source.structured_points.as_ref().map(|points| {
            let mut copy = Points::new();
            copy.shallow_copy(points);
            copy
        });
        self.data_set.shallow_copy(&source.data_set);
    }

    pub fn get_field_data(&self) -> &FieldData {
        self.data_set.data_object().get_field_data()
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

    /// VTK: `vtkImageData::SetExtent`.
    pub fn set_extent(&mut self, extent: [i32; 6]) {
        if self.extent == extent {
            return;
        }
        self.extent = extent;
        self.resize_existing_scalars();
        self.build_points();
    }

    /// VTK: `vtkImageData::GetExtent`.
    pub fn get_extent(&self) -> [i32; 6] {
        self.extent
    }

    /// VTK: `vtkImageData::SetDimensions`.
    pub fn set_dimensions(&mut self, dimensions: [i32; 3]) {
        let extent = extent_from_dimensions(dimensions);
        if self.extent == extent {
            return;
        }
        self.extent = extent;
        self.resize_existing_scalars();
        self.build_points();
    }

    /// VTK: `vtkImageData::GetDimensions`.
    pub fn get_dimensions(&self) -> [i32; 3] {
        StructuredData::get_dimensions_from_extent(self.extent)
    }

    /// VTK: `vtkImageData::SetOrigin`.
    pub fn set_origin(&mut self, origin: [f64; 3]) {
        if self.origin == origin {
            return;
        }
        self.origin = origin;
        self.build_points();
    }

    /// VTK: `vtkImageData::GetOrigin`.
    pub fn get_origin(&self) -> [f64; 3] {
        self.origin
    }

    /// VTK: `vtkImageData::SetSpacing`.
    pub fn set_spacing(&mut self, spacing: [f64; 3]) {
        if self.spacing == spacing {
            return;
        }
        self.spacing = spacing;
        self.build_points();
    }

    /// VTK: `vtkImageData::GetSpacing`.
    pub fn get_spacing(&self) -> [f64; 3] {
        self.spacing
    }

    /// VTK: `vtkImageData::SetDirectionMatrix`.
    pub fn set_direction_matrix(&mut self, elements: [f64; 9]) {
        if self.direction_matrix == elements {
            return;
        }
        self.direction_matrix = elements;
        self.build_points();
    }

    /// VTK: `vtkImageData::GetDirectionMatrix`.
    pub fn get_direction_matrix(&self) -> [f64; 9] {
        self.direction_matrix
    }

    /// VTK: `vtkImageData::ComputeTransforms`.
    pub fn compute_transforms(&self) -> ([f64; 16], [f64; 16]) {
        (
            Self::compute_index_to_physical_matrix(
                self.origin,
                self.spacing,
                self.direction_matrix,
            ),
            Self::compute_physical_to_index_matrix(
                self.origin,
                self.spacing,
                self.direction_matrix,
            ),
        )
    }

    /// VTK: `vtkImageData::ComputeIndexToPhysicalMatrix`.
    pub fn compute_index_to_physical_matrix(
        origin: [f64; 3],
        spacing: [f64; 3],
        direction: [f64; 9],
    ) -> [f64; 16] {
        [
            direction[0] * spacing[0],
            direction[1] * spacing[1],
            direction[2] * spacing[2],
            origin[0],
            direction[3] * spacing[0],
            direction[4] * spacing[1],
            direction[5] * spacing[2],
            origin[1],
            direction[6] * spacing[0],
            direction[7] * spacing[1],
            direction[8] * spacing[2],
            origin[2],
            0.0,
            0.0,
            0.0,
            1.0,
        ]
    }

    /// VTK: `vtkImageData::ComputePhysicalToIndexMatrix`.
    pub fn compute_physical_to_index_matrix(
        origin: [f64; 3],
        spacing: [f64; 3],
        direction: [f64; 9],
    ) -> [f64; 16] {
        let inverse_direction = invert_3x3(direction).unwrap_or([0.0; 9]);
        let inverse_origin =
            multiply_3x3_vec(inverse_direction, [-origin[0], -origin[1], -origin[2]]);

        let mut result = [0.0; 16];
        for axis in 0..3 {
            if spacing[axis] != 0.0 {
                result[axis * 4] = inverse_direction[axis * 3] / spacing[axis];
                result[axis * 4 + 1] = inverse_direction[axis * 3 + 1] / spacing[axis];
                result[axis * 4 + 2] = inverse_direction[axis * 3 + 2] / spacing[axis];
                result[axis * 4 + 3] = inverse_origin[axis] / spacing[axis];
            }
        }
        result[15] = 1.0;
        result
    }

    /// VTK: `vtkImageData::ApplyIndexToPhysicalMatrix`.
    pub fn apply_index_to_physical_matrix(&mut self, source_index_to_physical_matrix: [f64; 16]) {
        let origin = [
            source_index_to_physical_matrix[3],
            source_index_to_physical_matrix[7],
            source_index_to_physical_matrix[11],
        ];
        let mut spacing = [0.0; 3];
        let mut direction_matrix = [0.0; 9];

        for axis in 0..3 {
            let mut direction = [
                source_index_to_physical_matrix[axis],
                source_index_to_physical_matrix[4 + axis],
                source_index_to_physical_matrix[8 + axis],
            ];
            spacing[axis] = normalize_3(&mut direction);
            direction_matrix[axis] = direction[0];
            direction_matrix[3 + axis] = direction[1];
            direction_matrix[6 + axis] = direction[2];
        }

        if self.origin != origin
            || self.spacing != spacing
            || self.direction_matrix != direction_matrix
        {
            self.origin = origin;
            self.spacing = spacing;
            self.direction_matrix = direction_matrix;
            self.build_points();
        }
    }

    /// VTK: `vtkImageData::ApplyPhysicalToIndexMatrix`.
    pub fn apply_physical_to_index_matrix(&mut self, source_physical_to_index_matrix: [f64; 16]) {
        if let Some(index_to_physical_matrix) = invert_affine_4x4(source_physical_to_index_matrix) {
            self.apply_index_to_physical_matrix(index_to_physical_matrix);
        }
    }

    /// VTK: `vtkImageData::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        StructuredData::get_number_of_points(self.extent)
    }

    /// VTK: `vtkImageData::GetNumberOfCells`.
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

    /// VTK: `vtkCartesianGrid::GetPoint(vtkIdType, double*)`.
    pub fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        self.structured_points
            .as_ref()
            .map(|points| points.get_point(point_id))
            .unwrap_or_else(|| {
                let ijk = self
                    .ijk_from_point_id(point_id)
                    .expect("point id out of range");
                self.transform_index_to_physical_point(ijk)
            })
    }

    /// VTK: `vtkImageData::GetCell`.
    pub fn get_cell(&self, cell_id: VtkIdType) -> ImageCell {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            panic!("cell id out of range");
        }
        if !self.is_cell_visible(cell_id) {
            return ImageCell {
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

        ImageCell {
            cell_type: self.cell_type(),
            point_ids,
            points,
        }
    }

    /// VTK: inherited `vtkCartesianGrid::GetCell(int, int, int)`.
    pub fn get_cell_ijk(&self, i: i32, j: i32, k: i32) -> ImageCell {
        self.get_cell(self.compute_cell_id([i, j, k]))
    }

    /// VTK: `vtkCartesianGrid::GetCellType`.
    pub fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        if !self.is_cell_visible(cell_id) {
            return CellType::Empty as i32;
        }
        self.cell_type() as i32
    }

    /// VTK: `vtkCartesianGrid::GetCellSize`.
    pub fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType {
        if !self.is_cell_visible(cell_id) {
            return 0;
        }
        StructuredData::cell_size_for_extent(cell_id, self.extent)
    }

    /// VTK: `vtkCartesianGrid::GetCellPoints(vtkIdType, vtkIdList*)`.
    pub fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        point_ids.reset();
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        for point_id in StructuredData::cell_point_ids_for_extent(cell_id, self.extent, true) {
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

    /// VTK: `vtkImageData::GetCellBounds`.
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

    /// VTK: `vtkCartesianGrid::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        8
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

    /// VTK: `vtkDataSet::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            POINT => self.get_number_of_points(),
            CELL => self.get_number_of_cells(),
            _ => self.data_set.get_number_of_elements(attribute_type),
        }
    }

    /// VTK: `vtkImageData::GetNumberOfScalarComponents`.
    pub fn get_number_of_scalar_components(&self) -> i32 {
        self.scalar_data()
            .map_or(1, |array| array.get_number_of_components())
    }

    /// VTK: `vtkImageData::GetScalarSize`.
    pub fn get_scalar_size(&self) -> i32 {
        self.scalar_data_type().size() as i32
    }

    /// VTK: `vtkImageData::GetScalarTypeMin`.
    pub fn get_scalar_type_min(&self) -> f64 {
        self.scalar_data_type()
            .range()
            .map_or(0.0, |(min, _max)| min)
    }

    /// VTK: `vtkImageData::GetScalarTypeMax`.
    pub fn get_scalar_type_max(&self) -> f64 {
        self.scalar_data_type()
            .range()
            .map_or(1.0, |(_min, max)| max)
    }

    /// VTK: `vtkImageData::ComputeScalarRange`.
    pub fn compute_scalar_range(&self) -> [f64; 2] {
        let mut range = [VTK_DOUBLE_MAX, VTK_DOUBLE_MIN];

        if let Some(point_scalars) = self.get_point_data().get_field_data_scalars() {
            self.accumulate_visible_scalar_range(
                point_scalars,
                self.get_number_of_points(),
                |image, id| image.is_point_visible(id),
                &mut range,
            );
        }

        if let Some(cell_scalars) = self.get_cell_data().get_field_data_scalars() {
            self.accumulate_visible_scalar_range(
                cell_scalars,
                self.get_number_of_cells(),
                |image, id| image.is_cell_visible(id),
                &mut range,
            );
        }

        [
            if range[0] >= VTK_DOUBLE_MAX {
                0.0
            } else {
                range[0]
            },
            if range[1] <= VTK_DOUBLE_MIN {
                1.0
            } else {
                range[1]
            },
        ]
    }

    /// VTK: `vtkImageData::GetVoxelGradient`.
    pub fn get_voxel_gradient(
        &self,
        i: i32,
        j: i32,
        k: i32,
        scalars: &AnyArray,
        gradients: &mut AnyArray,
    ) {
        let mut tuple_id = 0_usize;
        for kk in 0..2 {
            for jj in 0..2 {
                for ii in 0..2 {
                    let gradient = self.get_point_gradient(i + ii, j + jj, k + kk, scalars);
                    let _ = gradients.insert_numeric_tuple_from_f64_checked(tuple_id, &gradient);
                    tuple_id += 1;
                }
            }
        }
    }

    /// VTK: `vtkImageData::GetPointGradient`.
    pub fn get_point_gradient(&self, i: i32, j: i32, k: i32, scalars: &AnyArray) -> [f64; 3] {
        let extent = self.get_extent();
        let dimensions = self.get_dimensions();

        let local_i = i - extent[0];
        let local_j = j - extent[2];
        let local_k = k - extent[4];

        if local_i < 0
            || local_i >= dimensions[0]
            || local_j < 0
            || local_j >= dimensions[1]
            || local_k < 0
            || local_k >= dimensions[2]
        {
            return [0.0; 3];
        }

        let dims = [
            VtkIdType::from(dimensions[0]),
            VtkIdType::from(dimensions[1]),
            VtkIdType::from(dimensions[2]),
        ];
        let ij_size = dims[0] * dims[1];
        let local = [
            VtkIdType::from(local_i),
            VtkIdType::from(local_j),
            VtkIdType::from(local_k),
        ];

        let mut gradient = [0.0; 3];
        for axis in 0..3 {
            gradient[axis] = self.gradient_axis_component(axis, local, dims, ij_size, scalars);
        }

        self.multiply_direction_3(gradient)
    }

    /// VTK: `vtkImageData::Crop`.
    pub fn crop(&mut self, update_extent: [i32; 6]) {
        let extent = self.get_extent();
        if extent_is_empty(extent) {
            return;
        }

        if extent == update_extent {
            return;
        }

        let new_extent = intersect_extents(update_extent, extent);
        if extent == new_extent {
            return;
        }

        if extent_is_empty(new_extent) {
            self.get_point_data_mut().initialize();
            self.get_cell_data_mut().initialize();
            self.set_extent(new_extent);
            return;
        }

        let number_of_points = StructuredData::get_number_of_points(new_extent) as usize;
        let number_of_cells = image_cell_count_for_crop(new_extent);

        let source_point_data = self.get_point_data().shallow_clone();
        let source_cell_data = self.get_cell_data().shallow_clone();

        let mut new_point_data = DataSetAttributes::new();
        let mut point_fields = DataSetAttributesFieldList::new();
        point_fields.initialize_field_list(&source_point_data);
        new_point_data.copy_allocate(&mut point_fields, number_of_points, 0);

        let point_inc_y = VtkIdType::from(extent[1] - extent[0] + 1);
        let point_inc_z = VtkIdType::from(extent[3] - extent[2] + 1) * point_inc_y;
        let mut out_id = 0_usize;
        let mut in_id_z = point_inc_z * VtkIdType::from(new_extent[4] - extent[4])
            + point_inc_y * VtkIdType::from(new_extent[2] - extent[2])
            + VtkIdType::from(new_extent[0] - extent[0]);

        for _idx_z in new_extent[4]..=new_extent[5] {
            let mut in_id_y = in_id_z;
            for _idx_y in new_extent[2]..=new_extent[3] {
                let mut in_id = in_id_y;
                for _idx_x in new_extent[0]..=new_extent[1] {
                    if let Ok(from_id) = usize::try_from(in_id) {
                        let _ = new_point_data.copy_data(
                            &point_fields,
                            0,
                            &source_point_data,
                            from_id,
                            out_id,
                        );
                    }
                    in_id += 1;
                    out_id += 1;
                }
                in_id_y += point_inc_y;
            }
            in_id_z += point_inc_z;
        }

        let mut new_cell_data = DataSetAttributes::new();
        let mut cell_fields = DataSetAttributesFieldList::new();
        cell_fields.initialize_field_list(&source_cell_data);
        new_cell_data.copy_allocate(&mut cell_fields, number_of_cells, 0);

        let max = [
            if new_extent[1] == new_extent[0] {
                new_extent[1] + 1
            } else {
                new_extent[1]
            },
            if new_extent[3] == new_extent[2] {
                new_extent[3] + 1
            } else {
                new_extent[3]
            },
            if new_extent[5] == new_extent[4] {
                new_extent[5] + 1
            } else {
                new_extent[5]
            },
        ];
        let cell_inc_y = VtkIdType::from(extent[1] - extent[0]);
        let cell_inc_z = VtkIdType::from(extent[3] - extent[2]) * cell_inc_y;
        out_id = 0;
        in_id_z = cell_inc_z * VtkIdType::from(new_extent[4] - extent[4])
            + cell_inc_y * VtkIdType::from(new_extent[2] - extent[2])
            + VtkIdType::from(new_extent[0] - extent[0]);

        for _idx_z in new_extent[4]..max[2] {
            let mut in_id_y = in_id_z;
            for _idx_y in new_extent[2]..max[1] {
                let mut in_id = in_id_y;
                for _idx_x in new_extent[0]..max[0] {
                    if let Ok(from_id) = usize::try_from(in_id) {
                        let _ = new_cell_data.copy_data(
                            &cell_fields,
                            0,
                            &source_cell_data,
                            from_id,
                            out_id,
                        );
                    }
                    in_id += 1;
                    out_id += 1;
                }
                in_id_y += cell_inc_y;
            }
            in_id_z += cell_inc_z;
        }

        self.get_point_data_mut().shallow_copy(&new_point_data);
        self.get_cell_data_mut().shallow_copy(&new_cell_data);
        self.set_extent(new_extent);
    }

    /// VTK: `vtkImageData::CopyAndCastFrom`.
    pub fn copy_and_cast_from(&mut self, in_data: &ImageData, extent: [i32; 6]) {
        let Some(in_scalars) = in_data.scalar_data() else {
            return;
        };
        if !in_scalars.is_numeric() {
            return;
        }

        let in_components = in_data.get_number_of_scalar_components();
        let out_components = self.get_number_of_scalar_components();
        if in_components <= 0 || out_components <= 0 {
            return;
        }

        let in_start_tuple = in_data.get_scalar_index_for_extent(extent);
        let out_start_tuple = self.get_scalar_index_for_extent(extent);
        if in_start_tuple < 0 || out_start_tuple < 0 {
            return;
        }

        let row_length =
            VtkIdType::from(extent[1] - extent[0] + 1) * VtkIdType::from(in_components);
        if row_length <= 0 {
            return;
        }

        let max_y = extent[3] - extent[2];
        let max_z = extent[5] - extent[4];
        if max_y < 0 || max_z < 0 {
            return;
        }

        let in_increments = in_data.get_continuous_increments(extent);
        let out_increments = self.get_continuous_increments(extent);
        let mut in_offset_z = in_start_tuple * VtkIdType::from(in_components);
        let mut out_offset_z = out_start_tuple * VtkIdType::from(out_components);

        let source_scalars = in_scalars.shallow_clone();
        let Some(out_scalars) = self.scalar_data_mut() else {
            return;
        };
        if !out_scalars.is_numeric() {
            return;
        }

        for _idx_z in 0..=max_z {
            let mut in_offset_y = in_offset_z;
            let mut out_offset_y = out_offset_z;
            for _idx_y in 0..=max_y {
                let mut in_offset = in_offset_y;
                let mut out_offset = out_offset_y;
                for _idx_r in 0..row_length {
                    let Some((in_tuple, in_component)) =
                        flat_scalar_offset_to_tuple_component(in_offset, in_components)
                    else {
                        return;
                    };
                    let Some((out_tuple, out_component)) =
                        flat_scalar_offset_to_tuple_component(out_offset, out_components)
                    else {
                        return;
                    };
                    let Ok(value) =
                        source_scalars.numeric_component_as_f64_checked(in_tuple, in_component)
                    else {
                        return;
                    };
                    if out_scalars
                        .set_numeric_component_from_f64_checked(out_tuple, out_component, value)
                        .is_err()
                    {
                        return;
                    }
                    in_offset += 1;
                    out_offset += 1;
                }
                in_offset_y += row_length + in_increments[1];
                out_offset_y += row_length + out_increments[1];
            }
            in_offset_z +=
                (row_length + in_increments[1]) * VtkIdType::from(max_y + 1) + in_increments[2];
            out_offset_z +=
                (row_length + out_increments[1]) * VtkIdType::from(max_y + 1) + out_increments[2];
        }
    }

    /// VTK: `vtkImageData::PrepareForNewData`.
    pub fn prepare_for_new_data(&mut self) {
        let scalars = self
            .get_point_data()
            .get_field_data_scalars()
            .map(FieldDataArray::shallow_clone);
        self.initialize();
        if let Some(scalars) = scalars {
            self.get_point_data_mut()
                .set_field_data_scalars(Some(scalars));
        }
    }

    /// VTK: `vtkImageData::AllocateScalars`.
    pub fn allocate_scalars(&mut self, data_type: i32, number_of_components: i32) {
        if data_type == VtkDataType::VTK_VOID || number_of_components < 1 {
            return;
        }
        let Some(data_type) = VtkDataType::from_id(data_type) else {
            return;
        };
        let Some(mut values) = AnyArray::create_array(data_type) else {
            return;
        };
        if !values.is_numeric() {
            return;
        }
        values.set_name("ImageScalars");
        values.set_number_of_components(number_of_components);
        values.set_number_of_tuples(self.get_number_of_points());
        self.get_point_data_mut()
            .set_field_data_scalars(Some(FieldDataArray::from_any_array(values)));
    }

    /// VTK: `vtkImageData::GetScalarComponentAsDouble`.
    pub fn get_scalar_component_as_double(&self, x: i32, y: i32, z: i32, component: i32) -> f64 {
        self.checked_scalar_component_as_double(x, y, z, component)
            .unwrap_or(0.0)
    }

    /// VTK: `vtkImageData::GetScalarComponentAsFloat`.
    pub fn get_scalar_component_as_float(&self, x: i32, y: i32, z: i32, component: i32) -> f32 {
        self.get_scalar_component_as_double(x, y, z, component) as f32
    }

    fn checked_scalar_component_as_double(
        &self,
        x: i32,
        y: i32,
        z: i32,
        component: i32,
    ) -> Option<f64> {
        if component < 0 || component >= self.get_number_of_scalar_components() {
            return None;
        }
        let scalars = self.scalar_data()?;
        let tuple = scalar_index_to_tuple(self.get_scalar_index(x, y, z))?;
        scalars
            .numeric_tuple_as_f64_checked(tuple)
            .ok()
            .and_then(|tuple| tuple.get(component as usize).copied())
    }

    /// VTK: `vtkImageData::SetScalarComponentFromDouble`.
    pub fn set_scalar_component_from_double(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        component: i32,
        value: f64,
    ) {
        if component < 0 || component >= self.get_number_of_scalar_components() {
            return;
        }
        let Some(tuple) = scalar_index_to_tuple(self.get_scalar_index(x, y, z)) else {
            return;
        };
        let Some(scalars) = self.scalar_data_mut() else {
            return;
        };
        let Ok(mut values) = scalars.numeric_tuple_as_f64_checked(tuple) else {
            return;
        };
        if let Some(slot) = values.get_mut(component as usize) {
            *slot = value;
            let _ = scalars.insert_numeric_tuple_from_f64_checked(tuple, &values);
        }
    }

    /// VTK: `vtkImageData::SetScalarComponentFromFloat`.
    pub fn set_scalar_component_from_float(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        component: i32,
        value: f32,
    ) {
        self.set_scalar_component_from_double(x, y, z, component, value as f64);
    }

    /// VTK: `vtkImageData::GetScalarIndex`.
    pub fn get_scalar_index(&self, x: i32, y: i32, z: i32) -> VtkIdType {
        let Some(scalars) = self.scalar_data() else {
            return -1;
        };
        if !self.contains_index([x, y, z]) {
            return -1;
        }
        let tuple = self.compute_point_id([x, y, z]);
        if tuple < 0 || tuple >= scalars.get_number_of_tuples() {
            return -1;
        }
        tuple
    }

    /// VTK: `vtkImageData::GetScalarIndex(int coordinate[3])`.
    pub fn get_scalar_index_for_coordinate(&self, coordinate: [i32; 3]) -> VtkIdType {
        self.get_scalar_index(coordinate[0], coordinate[1], coordinate[2])
    }

    /// VTK: `vtkImageData::GetScalarIndexForExtent`.
    pub fn get_scalar_index_for_extent(&self, extent: [i32; 6]) -> VtkIdType {
        self.get_scalar_index(extent[0], extent[2], extent[4])
    }

    /// VTK: `vtkImageData::ComputeIncrements(int, vtkIdType[3])`.
    pub fn compute_increments_for_components(&self, number_of_components: i32) -> [VtkIdType; 3] {
        compute_image_increments(self.extent, number_of_components)
    }

    /// VTK: `vtkImageData::ComputeIncrements(vtkDataArray*, vtkIdType[3])`.
    pub fn compute_increments_for_array(&self, scalars: Option<&AnyArray>) -> [VtkIdType; 3] {
        self.compute_increments_for_components(array_component_count_or_one(scalars))
    }

    /// VTK: `vtkImageData::ComputeIncrements(vtkIdType[3])`.
    pub fn compute_increments(&self) -> [VtkIdType; 3] {
        self.compute_increments_for_array(self.scalar_data())
    }

    /// VTK: `vtkImageData::GetIncrements`.
    pub fn get_increments(&self) -> [VtkIdType; 3] {
        self.compute_increments()
    }

    /// VTK: `vtkImageData::GetIncrements(vtkDataArray*)`.
    pub fn get_increments_for_array(&self, scalars: Option<&AnyArray>) -> [VtkIdType; 3] {
        self.compute_increments_for_array(scalars)
    }

    /// VTK: `vtkImageData::GetContinuousIncrements`.
    pub fn get_continuous_increments(&self, extent: [i32; 6]) -> [VtkIdType; 3] {
        self.get_continuous_increments_for_array(self.scalar_data(), extent)
    }

    /// VTK: `vtkImageData::GetContinuousIncrements(vtkDataArray*, int[6], ...)`.
    pub fn get_continuous_increments_for_array(
        &self,
        scalars: Option<&AnyArray>,
        extent: [i32; 6],
    ) -> [VtkIdType; 3] {
        let e0 = extent[0].max(self.extent[0]);
        let e1 = extent[1].min(self.extent[1]);
        let e2 = extent[2].max(self.extent[2]);
        let e3 = extent[3].min(self.extent[3]);
        let inc = self.compute_increments_for_array(scalars);
        [
            0,
            inc[1] - VtkIdType::from(e1 - e0 + 1) * inc[0],
            inc[2] - VtkIdType::from(e3 - e2 + 1) * inc[1],
        ]
    }

    /// VTK: `vtkImageData::SetAxisUpdateExtent`.
    pub fn set_axis_update_extent(
        &self,
        axis: i32,
        min: i32,
        max: i32,
        update_extent: [i32; 6],
        axis_update_extent: &mut [i32; 6],
    ) {
        if !(0..=2).contains(&axis) {
            return;
        }

        *axis_update_extent = update_extent;
        let axis = axis as usize;
        axis_update_extent[axis * 2] = min;
        axis_update_extent[axis * 2 + 1] = max;
    }

    /// VTK: `vtkImageData::GetAxisUpdateExtent`.
    pub fn get_axis_update_extent(
        &self,
        axis: i32,
        min: &mut i32,
        max: &mut i32,
        update_extent: [i32; 6],
    ) {
        if !(0..=2).contains(&axis) {
            return;
        }

        let axis = axis as usize;
        *min = update_extent[axis * 2];
        *max = update_extent[axis * 2 + 1];
    }

    /// VTK: `vtkImageData::GetArrayIncrements`.
    pub fn get_array_increments(&self, array: &AnyArray) -> [VtkIdType; 3] {
        self.compute_increments_for_components(array.get_number_of_components())
    }

    /// VTK: `vtkImageData::ComputePointId`.
    pub fn compute_point_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_point_id_for_extent(self.extent, ijk)
    }

    /// VTK: `vtkImageData::ComputeCellId`.
    pub fn compute_cell_id(&self, ijk: [i32; 3]) -> VtkIdType {
        StructuredData::compute_cell_id_for_extent(self.extent, ijk)
    }

    /// VTK: `vtkImageData::FindPoint`.
    ///
    /// Returns `-1` for points outside the extent or for zero spacing on an
    /// axis with more than one point.
    pub fn find_point(&self, point: [f64; 3]) -> VtkIdType {
        let dimensions = self.get_dimensions();
        let mut ijk = [0_i32; 3];
        let continuous_index = self.transform_physical_point_to_continuous_index(point);
        for axis in 0..3 {
            if self.spacing[axis] == 0.0 && dimensions[axis] > 1 {
                return -1;
            }
            ijk[axis] = (continuous_index[axis] + 0.5).floor() as i32;
        }
        if !self.contains_index(ijk) {
            return -1;
        }
        self.compute_point_id(ijk)
    }

    /// VTK: `vtkImageData::ComputeStructuredCoordinates`.
    pub fn compute_structured_coordinates(&self, point: [f64; 3]) -> (bool, [i32; 3], [f64; 3]) {
        self.compute_structured_coordinates_with_tolerance(point, 1e-12)
    }

    /// VTK: `vtkImageData::ComputeStructuredCoordinates(..., double tol2)`.
    pub fn compute_structured_coordinates_with_tolerance(
        &self,
        point: [f64; 3],
        tolerance_squared: f64,
    ) -> (bool, [i32; 3], [f64; 3]) {
        let continuous_index = self.transform_physical_point_to_continuous_index(point);
        let mut ijk = [0; 3];
        let mut pcoords = [0.0; 3];
        let mut is_in_bounds = true;

        for axis in 0..3 {
            ijk[axis] = continuous_index[axis].floor() as i32;
            pcoords[axis] = continuous_index[axis] - f64::from(ijk[axis]);

            let min_extent = self.extent[axis * 2];
            let max_extent = self.extent[axis * 2 + 1];
            let mut axis_in_bounds = false;

            if min_extent == max_extent || ijk[axis] < min_extent {
                let distance = continuous_index[axis] - f64::from(min_extent);
                if distance * distance <= tolerance_squared {
                    pcoords[axis] = 0.0;
                    ijk[axis] = min_extent;
                    axis_in_bounds = true;
                }
            } else if ijk[axis] >= max_extent {
                let distance = continuous_index[axis] - f64::from(max_extent);
                if distance * distance <= tolerance_squared {
                    pcoords[axis] = 1.0;
                    ijk[axis] = max_extent - 1;
                    axis_in_bounds = true;
                }
            } else {
                axis_in_bounds = true;
            }

            is_in_bounds &= axis_in_bounds;
        }

        (is_in_bounds, ijk, pcoords)
    }

    /// VTK: `vtkImageData::FindCell`.
    pub fn find_cell(
        &self,
        point: [f64; 3],
        tolerance_squared: f64,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: Option<&mut [f64]>,
    ) -> VtkIdType {
        let (in_bounds, mut idx, mut local_pcoords) = self.compute_structured_coordinates(point);

        if !in_bounds {
            let mut distance_squared = 0.0;

            for axis in 0..3 {
                let min_idx = self.extent[axis * 2];
                let max_idx = self.extent[axis * 2 + 1];

                if idx[axis] < min_idx {
                    let distance = (f64::from(idx[axis]) + local_pcoords[axis]
                        - f64::from(min_idx))
                        * self.spacing[axis];
                    idx[axis] = min_idx;
                    local_pcoords[axis] = 0.0;
                    distance_squared += distance * distance;
                } else if idx[axis] >= max_idx {
                    let distance = (f64::from(idx[axis]) + local_pcoords[axis]
                        - f64::from(max_idx))
                        * self.spacing[axis];
                    if max_idx == min_idx {
                        idx[axis] = min_idx;
                        local_pcoords[axis] = 0.0;
                    } else {
                        idx[axis] = max_idx - 1;
                        local_pcoords[axis] = 1.0;
                    }
                    distance_squared += distance * distance;
                }
            }

            if distance_squared > tolerance_squared {
                *pcoords = local_pcoords;
                return -1;
            }
        }

        if weights.is_some() {
            match StructuredData::get_data_description_from_extent(self.extent) {
                VTK_STRUCTURED_XZ_PLANE => {
                    local_pcoords[1] = local_pcoords[2];
                    local_pcoords[2] = 0.0;
                }
                VTK_STRUCTURED_YZ_PLANE => {
                    local_pcoords[0] = local_pcoords[1];
                    local_pcoords[1] = local_pcoords[2];
                    local_pcoords[2] = 0.0;
                }
                VTK_STRUCTURED_XY_PLANE => {
                    local_pcoords[2] = 0.0;
                }
                _ => {}
            }
        }

        *pcoords = local_pcoords;

        if let Some(weights) = weights {
            voxel_interpolation_functions(local_pcoords, weights);
        }

        *sub_id = 0;
        let cell_id = self.compute_cell_id(idx);
        if !self.is_cell_visible(cell_id) {
            return -1;
        }
        cell_id
    }

    /// VTK: `vtkImageData::ComputeBounds`.
    pub fn compute_bounds(&mut self) {
        let bounds = self.compute_bounds_value();
        self.data_set.set_bounds(bounds);
    }

    /// VTK: `vtkImageData::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.data_set.get_actual_memory_size()
    }

    /// VTK: `vtkImageData::PrintSelf`.
    pub fn print_self(&self) -> String {
        let increments = self.compute_increments();
        format!(
            "vtkImageData\n  Spacing: ({}, {}, {})\n  Origin: ({}, {}, {})\n  Direction: ({}, {}, {}, {}, {}, {}, {}, {}, {})\n  Increments: ({}, {}, {})",
            self.spacing[0],
            self.spacing[1],
            self.spacing[2],
            self.origin[0],
            self.origin[1],
            self.origin[2],
            self.direction_matrix[0],
            self.direction_matrix[1],
            self.direction_matrix[2],
            self.direction_matrix[3],
            self.direction_matrix[4],
            self.direction_matrix[5],
            self.direction_matrix[6],
            self.direction_matrix[7],
            self.direction_matrix[8],
            increments[0],
            increments[1],
            increments[2],
        )
    }

    /// VTK: `vtkImageData::ComputeInternalExtent`.
    pub fn compute_internal_extent(&self, target_extent: [i32; 6], boundary: [i32; 6]) -> [i32; 6] {
        let mut internal_extent = [0; 6];
        for axis in 0..3 {
            internal_extent[axis * 2] = target_extent[axis * 2];
            if internal_extent[axis * 2] - boundary[axis * 2] < self.extent[axis * 2] {
                internal_extent[axis * 2] = self.extent[axis * 2] + boundary[axis * 2];
            }

            internal_extent[axis * 2 + 1] = target_extent[axis * 2 + 1];
            if internal_extent[axis * 2 + 1] + boundary[axis * 2 + 1] > self.extent[axis * 2 + 1] {
                internal_extent[axis * 2 + 1] = self.extent[axis * 2 + 1] - boundary[axis * 2 + 1];
            }
        }
        internal_extent
    }

    fn compute_bounds_value(&self) -> BoundingBox {
        if self.is_empty() {
            return BoundingBox::empty();
        }

        let corners = [
            [self.extent[0], self.extent[2], self.extent[4]],
            [self.extent[1], self.extent[2], self.extent[4]],
            [self.extent[0], self.extent[3], self.extent[4]],
            [self.extent[1], self.extent[3], self.extent[4]],
            [self.extent[0], self.extent[2], self.extent[5]],
            [self.extent[1], self.extent[2], self.extent[5]],
            [self.extent[0], self.extent[3], self.extent[5]],
            [self.extent[1], self.extent[3], self.extent[5]],
        ];
        let mut bounds = BoundingBox::empty();
        for corner in corners {
            bounds.add_point(self.transform_index_to_physical_point(corner));
        }
        bounds
    }

    /// VTK: `vtkImageData::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.compute_bounds_value().get_bounds()
    }

    /// VTK: `vtkImageData::TransformIndexToPhysicalPoint`.
    pub fn transform_index_to_physical_point(&self, ijk: [i32; 3]) -> [f64; 3] {
        self.transform_continuous_index_to_physical_point([
            ijk[0] as f64,
            ijk[1] as f64,
            ijk[2] as f64,
        ])
    }

    /// VTK: `vtkImageData::TransformContinuousIndexToPhysicalPoint`.
    pub fn transform_continuous_index_to_physical_point(&self, ijk: [f64; 3]) -> [f64; 3] {
        [
            ijk[0] * self.spacing[0] * self.direction_matrix[0]
                + ijk[1] * self.spacing[1] * self.direction_matrix[1]
                + ijk[2] * self.spacing[2] * self.direction_matrix[2]
                + self.origin[0],
            ijk[0] * self.spacing[0] * self.direction_matrix[3]
                + ijk[1] * self.spacing[1] * self.direction_matrix[4]
                + ijk[2] * self.spacing[2] * self.direction_matrix[5]
                + self.origin[1],
            ijk[0] * self.spacing[0] * self.direction_matrix[6]
                + ijk[1] * self.spacing[1] * self.direction_matrix[7]
                + ijk[2] * self.spacing[2] * self.direction_matrix[8]
                + self.origin[2],
        ]
    }

    /// VTK: `vtkImageData::TransformPhysicalPointToContinuousIndex`.
    pub fn transform_physical_point_to_continuous_index(&self, xyz: [f64; 3]) -> [f64; 3] {
        let delta = [
            xyz[0] - self.origin[0],
            xyz[1] - self.origin[1],
            xyz[2] - self.origin[2],
        ];
        let Some(inverse_direction) = invert_3x3(self.direction_matrix) else {
            return [f64::NAN; 3];
        };
        let rotated = multiply_3x3_vec(inverse_direction, delta);
        [
            if self.spacing[0] == 0.0 {
                self.extent[0] as f64
            } else {
                rotated[0] / self.spacing[0]
            },
            if self.spacing[1] == 0.0 {
                self.extent[2] as f64
            } else {
                rotated[1] / self.spacing[1]
            },
            if self.spacing[2] == 0.0 {
                self.extent[4] as f64
            } else {
                rotated[2] / self.spacing[2]
            },
        ]
    }

    /// VTK: `vtkImageData::TransformPhysicalNormalToContinuousIndex`.
    pub fn transform_physical_normal_to_continuous_index(&self, normal: [f64; 3]) -> [f64; 3] {
        let matrix = Self::compute_index_to_physical_matrix(
            self.origin,
            self.spacing,
            self.direction_matrix,
        );
        transform_normal_with_matrix(normal, matrix)
    }

    /// VTK: `vtkImageData::TransformPhysicalPlaneToContinuousIndex`.
    pub fn transform_physical_plane_to_continuous_index(&self, plane: [f64; 4]) -> [f64; 4] {
        let mut transformed_normal =
            self.transform_physical_normal_to_continuous_index([plane[0], plane[1], plane[2]]);
        normalize_3(&mut transformed_normal);

        let transformed_point = self.transform_physical_point_to_continuous_index([
            -plane[3] * plane[0],
            -plane[3] * plane[1],
            -plane[3] * plane[2],
        ]);

        [
            transformed_normal[0],
            transformed_normal[1],
            transformed_normal[2],
            -transformed_normal[0] * transformed_point[0]
                - transformed_normal[1] * transformed_point[1]
                - transformed_normal[2] * transformed_point[2],
        ]
    }

    fn scalar_data(&self) -> Option<&AnyArray> {
        self.get_point_data()
            .get_field_data_scalars()
            .map(FieldDataArray::get_data)
    }

    fn scalar_data_mut(&mut self) -> Option<&mut AnyArray> {
        self.get_point_data_mut()
            .get_scalars_mut()
            .map(FieldDataArray::get_data_mut)
    }

    fn scalar_data_type(&self) -> VtkDataType {
        self.scalar_data()
            .map_or(VtkDataType::Double, AnyArray::get_data_type)
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
            if !value.is_nan() {
                range[0] = range[0].min(value);
                range[1] = range[1].max(value);
            }
        }
    }

    fn gradient_axis_component(
        &self,
        axis: usize,
        local: [VtkIdType; 3],
        dimensions: [VtkIdType; 3],
        ij_size: VtkIdType,
        scalars: &AnyArray,
    ) -> f64 {
        if dimensions[axis] == 1 {
            return 0.0;
        }

        let mut plus = local;
        let mut minus = local;
        let scale = if local[axis] == 0 {
            plus[axis] += 1;
            1.0
        } else if local[axis] == dimensions[axis] - 1 {
            minus[axis] -= 1;
            1.0
        } else {
            plus[axis] += 1;
            minus[axis] -= 1;
            0.5
        };

        let sp = numeric_component_zero(scalars, image_local_tuple_id(plus, dimensions, ij_size));
        let sm = numeric_component_zero(scalars, image_local_tuple_id(minus, dimensions, ij_size));
        scale * (sm - sp) / self.spacing[axis]
    }

    fn multiply_direction_3(&self, vector: [f64; 3]) -> [f64; 3] {
        multiply_3x3_vec(self.direction_matrix, vector)
    }

    fn is_empty(&self) -> bool {
        self.extent[0] > self.extent[1]
            || self.extent[2] > self.extent[3]
            || self.extent[4] > self.extent[5]
    }

    fn contains_index(&self, ijk: [i32; 3]) -> bool {
        !self.is_empty()
            && ijk[0] >= self.extent[0]
            && ijk[0] <= self.extent[1]
            && ijk[1] >= self.extent[2]
            && ijk[1] <= self.extent[3]
            && ijk[2] >= self.extent[4]
            && ijk[2] <= self.extent[5]
    }

    fn ijk_from_point_id(&self, point_id: VtkIdType) -> Option<[i32; 3]> {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return None;
        }
        Some(StructuredData::compute_point_structured_coords_for_extent(
            point_id,
            self.extent,
        ))
    }

    fn cell_type(&self) -> CellType {
        StructuredData::cell_type_for_extent(self.extent, true)
    }

    fn resize_existing_scalars(&mut self) {
        let number_of_points = self.get_number_of_points();
        let len = self.scalar_data().and_then(|scalars| {
            number_of_points.checked_mul(scalars.get_number_of_components() as VtkIdType)
        });
        if let (Some(scalars), Some(len)) = (self.scalar_data_mut(), len) {
            let components = scalars.get_number_of_components() as VtkIdType;
            scalars.set_number_of_tuples(len / components);
        }
    }

    /// VTK: `vtkImageData::BuildPoints`.
    fn build_points(&mut self) {
        self.ensure_structured_points_storage();

        let dimensions = self.get_dimensions();
        let extent = self.extent;
        let [x_coordinates, y_coordinates, z_coordinates] = self.axis_coordinate_arrays(dimensions);
        let data_description = StructuredData::get_data_description_from_extent(extent);
        let number_of_points = StructuredData::get_number_of_points(extent);

        let points = self
            .structured_points
            .as_mut()
            .expect("structured points storage was created");
        let AnyArray::StructuredPoint(point_array) = points.get_data_mut() else {
            panic!("GetPoints()->GetData() is not a vtkStructuredPointArray");
        };
        point_array.construct_backend(
            &x_coordinates,
            &y_coordinates,
            &z_coordinates,
            extent,
            data_description,
            self.direction_matrix,
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

    fn axis_coordinate_arrays(&self, dimensions: [i32; 3]) -> [AnyArray; 3] {
        [
            image_axis_coordinates(
                self.origin[0],
                self.spacing[0],
                self.extent[0],
                dimensions[0],
                self.direction_matrix_is_identity(),
            ),
            image_axis_coordinates(
                self.origin[1],
                self.spacing[1],
                self.extent[2],
                dimensions[1],
                self.direction_matrix_is_identity(),
            ),
            image_axis_coordinates(
                self.origin[2],
                self.spacing[2],
                self.extent[4],
                dimensions[2],
                self.direction_matrix_is_identity(),
            ),
        ]
    }

    fn direction_matrix_is_identity(&self) -> bool {
        self.direction_matrix == identity_direction()
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

fn image_axis_coordinates(
    origin: f64,
    spacing: f64,
    extent_min: i32,
    dimension: i32,
    direction_matrix_is_identity: bool,
) -> AnyArray {
    let values = if direction_matrix_is_identity {
        (0..dimension.max(0))
            .map(|loc| origin + spacing * f64::from(extent_min + loc))
            .collect()
    } else {
        vec![origin, origin + spacing]
    };
    AnyArray::Double(DoubleArray::from_vec("", values, 1))
}

impl DataSetApi for ImageData {
    fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    fn data_set_mut(&mut self) -> &mut DataSet {
        &mut self.data_set
    }

    fn get_class_name(&self) -> &'static str {
        "vtkImageData"
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        ImageData::get_number_of_cells(self)
    }

    fn get_number_of_points(&self) -> VtkIdType {
        ImageData::get_number_of_points(self)
    }

    fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        ImageData::get_cell_type(self, cell_id)
    }

    fn get_cell_points(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        ImageData::get_cell_points(self, cell_id, point_ids);
    }

    fn get_point(&self, point_id: VtkIdType) -> [f64; 3] {
        ImageData::get_point(self, point_id)
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

fn image_cell_count_for_crop(extent: [i32; 6]) -> usize {
    let mut number_of_cells = 1_usize;
    for axis in 0..3 {
        let cells = (extent[axis * 2 + 1] - extent[axis * 2]).max(1);
        number_of_cells = number_of_cells.saturating_mul(cells as usize);
    }
    number_of_cells
}

fn scalar_index_to_tuple(index: VtkIdType) -> Option<usize> {
    usize::try_from(index).ok()
}

fn image_local_tuple_id(
    local: [VtkIdType; 3],
    dimensions: [VtkIdType; 3],
    ij_size: VtkIdType,
) -> VtkIdType {
    local[0] + local[1] * dimensions[0] + local[2] * ij_size
}

fn numeric_component_zero(array: &AnyArray, tuple_id: VtkIdType) -> f64 {
    let Some(tuple_id) = scalar_index_to_tuple(tuple_id) else {
        return 0.0;
    };
    array
        .numeric_tuple_as_f64_checked(tuple_id)
        .ok()
        .and_then(|tuple| tuple.first().copied())
        .unwrap_or(0.0)
}

fn flat_scalar_offset_to_tuple_component(
    offset: VtkIdType,
    number_of_components: i32,
) -> Option<(usize, usize)> {
    if offset < 0 || number_of_components <= 0 {
        return None;
    }
    let components = VtkIdType::from(number_of_components);
    Some((
        usize::try_from(offset / components).ok()?,
        usize::try_from(offset % components).ok()?,
    ))
}

fn array_component_count_or_one(array: Option<&AnyArray>) -> i32 {
    array.map_or(1, AnyArray::get_number_of_components)
}

fn compute_image_increments(extent: [i32; 6], number_of_components: i32) -> [VtkIdType; 3] {
    let mut increments = [0; 3];
    let mut increment = VtkIdType::from(number_of_components);
    for axis in 0..3 {
        increments[axis] = increment;
        increment *= VtkIdType::from(extent[axis * 2 + 1] - extent[axis * 2] + 1);
    }
    increments
}

fn voxel_interpolation_functions(pcoords: [f64; 3], weights: &mut [f64]) {
    if weights.len() < 8 {
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

fn identity_direction() -> [f64; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

fn multiply_3x3_vec(matrix: [f64; 9], vector: [f64; 3]) -> [f64; 3] {
    [
        matrix[0] * vector[0] + matrix[1] * vector[1] + matrix[2] * vector[2],
        matrix[3] * vector[0] + matrix[4] * vector[1] + matrix[5] * vector[2],
        matrix[6] * vector[0] + matrix[7] * vector[1] + matrix[8] * vector[2],
    ]
}

fn normalize_3(vector: &mut [f64; 3]) -> f64 {
    let norm = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if norm != 0.0 {
        vector[0] /= norm;
        vector[1] /= norm;
        vector[2] /= norm;
    }
    norm
}

fn transform_normal_with_matrix(normal: [f64; 3], matrix: [f64; 16]) -> [f64; 3] {
    [
        matrix[0] * normal[0] + matrix[4] * normal[1] + matrix[8] * normal[2],
        matrix[1] * normal[0] + matrix[5] * normal[1] + matrix[9] * normal[2],
        matrix[2] * normal[0] + matrix[6] * normal[1] + matrix[10] * normal[2],
    ]
}

fn invert_3x3(matrix: [f64; 9]) -> Option<[f64; 9]> {
    let det = matrix[0] * (matrix[4] * matrix[8] - matrix[5] * matrix[7])
        - matrix[1] * (matrix[3] * matrix[8] - matrix[5] * matrix[6])
        + matrix[2] * (matrix[3] * matrix[7] - matrix[4] * matrix[6]);
    if det == 0.0 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        (matrix[4] * matrix[8] - matrix[5] * matrix[7]) * inv_det,
        (matrix[2] * matrix[7] - matrix[1] * matrix[8]) * inv_det,
        (matrix[1] * matrix[5] - matrix[2] * matrix[4]) * inv_det,
        (matrix[5] * matrix[6] - matrix[3] * matrix[8]) * inv_det,
        (matrix[0] * matrix[8] - matrix[2] * matrix[6]) * inv_det,
        (matrix[2] * matrix[3] - matrix[0] * matrix[5]) * inv_det,
        (matrix[3] * matrix[7] - matrix[4] * matrix[6]) * inv_det,
        (matrix[1] * matrix[6] - matrix[0] * matrix[7]) * inv_det,
        (matrix[0] * matrix[4] - matrix[1] * matrix[3]) * inv_det,
    ])
}

fn invert_affine_4x4(matrix: [f64; 16]) -> Option<[f64; 16]> {
    let linear = [
        matrix[0], matrix[1], matrix[2], matrix[4], matrix[5], matrix[6], matrix[8], matrix[9],
        matrix[10],
    ];
    let inverse_linear = invert_3x3(linear)?;
    let translation = [matrix[3], matrix[7], matrix[11]];
    let inverse_translation = multiply_3x3_vec(
        inverse_linear,
        [-translation[0], -translation[1], -translation[2]],
    );

    Some([
        inverse_linear[0],
        inverse_linear[1],
        inverse_linear[2],
        inverse_translation[0],
        inverse_linear[3],
        inverse_linear[4],
        inverse_linear[5],
        inverse_translation[1],
        inverse_linear[6],
        inverse_linear[7],
        inverse_linear[8],
        inverse_translation[2],
        0.0,
        0.0,
        0.0,
        1.0,
    ])
}
