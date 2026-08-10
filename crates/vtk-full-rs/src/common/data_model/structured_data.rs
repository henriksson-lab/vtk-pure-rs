use crate::common::core::{
    AnyArray, IdList, IntConstantArray, Points, StructuredPointArray, UnsignedCharConstantArray,
    VtkIdType,
};
use crate::common::data_model::{
    CellType, StructuredCellArray, StructuredExtent, HIDDENCELL, HIDDENPOINT, REFINEDCELL,
};

/// Static helper namespace for topologically regular datasets.
///
/// VTK origin: `VTK/Common/DataModel/vtkStructuredData.h` and
/// `VTK/Common/DataModel/vtkStructuredData.cxx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuredData;

/// VTK origin: `vtkStructuredData::vtkStructuredDataType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StructuredDataType {
    Invalid = -1,
    Unchanged = 0,
    SinglePoint = 1,
    XLine = 2,
    YLine = 3,
    ZLine = 4,
    XYPlane = 5,
    YZPlane = 6,
    XZPlane = 7,
    XYZGrid = 8,
    Empty = 9,
}

pub const VTK_STRUCTURED_INVALID: i32 = -1;
pub const VTK_STRUCTURED_UNCHANGED: i32 = 0;
pub const VTK_STRUCTURED_SINGLE_POINT: i32 = 1;
pub const VTK_STRUCTURED_X_LINE: i32 = 2;
pub const VTK_STRUCTURED_Y_LINE: i32 = 3;
pub const VTK_STRUCTURED_Z_LINE: i32 = 4;
pub const VTK_STRUCTURED_XY_PLANE: i32 = 5;
pub const VTK_STRUCTURED_YZ_PLANE: i32 = 6;
pub const VTK_STRUCTURED_XZ_PLANE: i32 = 7;
pub const VTK_STRUCTURED_XYZ_GRID: i32 = 8;
pub const VTK_STRUCTURED_EMPTY: i32 = 9;

impl StructuredData {
    /// VTK: `vtkStructuredData::SetDimensions`.
    pub fn set_dimensions(in_dim: [i32; 3], dim: &mut [i32; 3]) -> i32 {
        if in_dim == *dim {
            return VTK_STRUCTURED_UNCHANGED;
        }

        *dim = in_dim;
        if in_dim.iter().any(|&value| value < 1) {
            return VTK_STRUCTURED_EMPTY;
        }

        match [in_dim[0] > 1, in_dim[1] > 1, in_dim[2] > 1] {
            [true, true, true] => VTK_STRUCTURED_XYZ_GRID,
            [false, true, true] => VTK_STRUCTURED_YZ_PLANE,
            [true, false, true] => VTK_STRUCTURED_XZ_PLANE,
            [true, true, false] => VTK_STRUCTURED_XY_PLANE,
            [true, false, false] => VTK_STRUCTURED_X_LINE,
            [false, true, false] => VTK_STRUCTURED_Y_LINE,
            [false, false, true] => VTK_STRUCTURED_Z_LINE,
            [false, false, false] => VTK_STRUCTURED_SINGLE_POINT,
        }
    }

    /// VTK: `vtkStructuredData::SetExtent`.
    pub fn set_extent(in_ext: [i32; 6], ext: &mut [i32; 6]) -> i32 {
        if in_ext == *ext {
            return VTK_STRUCTURED_UNCHANGED;
        }

        *ext = in_ext;
        if in_ext[0] > in_ext[1] || in_ext[2] > in_ext[3] || in_ext[4] > in_ext[5] {
            return VTK_STRUCTURED_EMPTY;
        }

        match [
            in_ext[0] < in_ext[1],
            in_ext[2] < in_ext[3],
            in_ext[4] < in_ext[5],
        ] {
            [true, true, true] => VTK_STRUCTURED_XYZ_GRID,
            [false, true, true] => VTK_STRUCTURED_YZ_PLANE,
            [true, false, true] => VTK_STRUCTURED_XZ_PLANE,
            [true, true, false] => VTK_STRUCTURED_XY_PLANE,
            [true, false, false] => VTK_STRUCTURED_X_LINE,
            [false, true, false] => VTK_STRUCTURED_Y_LINE,
            [false, false, true] => VTK_STRUCTURED_Z_LINE,
            [false, false, false] => VTK_STRUCTURED_SINGLE_POINT,
        }
    }

    /// VTK: `vtkStructuredData::GetDataDescription`.
    pub fn get_data_description(dims: [i32; 3]) -> i32 {
        let mut temp_dims = [dims[0] + 1, dims[1] + 1, dims[2] + 1];
        Self::set_dimensions(dims, &mut temp_dims)
    }

    /// VTK: `vtkStructuredData::GetDataDescriptionFromExtent`.
    pub fn get_data_description_from_extent(ext: [i32; 6]) -> i32 {
        Self::get_data_description(Self::get_dimensions_from_extent(ext))
    }

    /// VTK: `vtkStructuredData::GetDataDimension(int dataDescription)`.
    pub fn get_data_dimension(data_description: i32) -> i32 {
        match data_description {
            VTK_STRUCTURED_EMPTY | VTK_STRUCTURED_SINGLE_POINT => 0,
            VTK_STRUCTURED_X_LINE | VTK_STRUCTURED_Y_LINE | VTK_STRUCTURED_Z_LINE => 1,
            VTK_STRUCTURED_XY_PLANE | VTK_STRUCTURED_YZ_PLANE | VTK_STRUCTURED_XZ_PLANE => 2,
            VTK_STRUCTURED_XYZ_GRID => 3,
            _ => -1,
        }
    }

    /// VTK: `vtkStructuredData::GetDataDimension(int ext[6])`.
    pub fn get_data_dimension_from_extent(ext: [i32; 6]) -> i32 {
        Self::get_data_dimension(Self::get_data_description_from_extent(ext))
    }

    /// VTK: `vtkStructuredData::GetNumberOfPoints`.
    pub fn get_number_of_points(ext: [i32; 6]) -> VtkIdType {
        VtkIdType::from(ext[1] - ext[0] + 1)
            * VtkIdType::from(ext[3] - ext[2] + 1)
            * VtkIdType::from(ext[5] - ext[4] + 1)
    }

    /// VTK: `vtkStructuredData::GetNumberOfCells`.
    pub fn get_number_of_cells(ext: [i32; 6]) -> VtkIdType {
        let dims = Self::get_dimensions_from_extent(ext);
        let cell_dims = [
            if dims[0] != 0 {
                (dims[0] - 1).max(1)
            } else {
                0
            },
            if dims[1] != 0 {
                (dims[1] - 1).max(1)
            } else {
                0
            },
            if dims[2] != 0 {
                (dims[2] - 1).max(1)
            } else {
                0
            },
        ];
        VtkIdType::from(cell_dims[0])
            * VtkIdType::from(cell_dims[1])
            * VtkIdType::from(cell_dims[2])
    }

    /// VTK: `vtkStructuredData::GetCellExtentFromPointExtent`.
    pub fn get_cell_extent_from_point_extent(node_extent: [i32; 6]) -> [i32; 6] {
        [
            node_extent[0],
            node_extent[0].max(node_extent[1] - 1),
            node_extent[2],
            node_extent[2].max(node_extent[3] - 1),
            node_extent[4],
            node_extent[4].max(node_extent[5] - 1),
        ]
    }

    /// VTK: `vtkStructuredData::GetDimensionsFromExtent`.
    pub fn get_dimensions_from_extent(ext: [i32; 6]) -> [i32; 3] {
        StructuredExtent::get_dimensions(ext)
    }

    /// VTK: `vtkStructuredData::GetCellDimensionsFromExtent`.
    pub fn get_cell_dimensions_from_extent(ext: [i32; 6]) -> [i32; 3] {
        [
            (ext[1] - ext[0]).max(0),
            (ext[3] - ext[2]).max(0),
            (ext[5] - ext[4]).max(0),
        ]
    }

    /// VTK: `vtkStructuredData::GetCellDimensionsFromPointDimensions`.
    pub fn get_cell_dimensions_from_point_dimensions(pntdims: [i32; 3]) -> [i32; 3] {
        [
            (pntdims[0] - 1).max(0),
            (pntdims[1] - 1).max(0),
            (pntdims[2] - 1).max(0),
        ]
    }

    /// VTK: `vtkStructuredData::GetLocalStructuredCoordinates`.
    pub fn get_local_structured_coordinates(ijk: [i32; 3], ext: [i32; 6]) -> [i32; 3] {
        [ijk[0] - ext[0], ijk[1] - ext[2], ijk[2] - ext[4]]
    }

    /// VTK: `vtkStructuredData::GetGlobalStructuredCoordinates`.
    pub fn get_global_structured_coordinates(lijk: [i32; 3], ext: [i32; 6]) -> [i32; 3] {
        [ext[0] + lijk[0], ext[2] + lijk[1], ext[4] + lijk[2]]
    }

    /// VTK: `vtkStructuredData::GetCellArray`.
    pub fn get_cell_array(
        extent: [i32; 6],
        use_pixel_voxel_orientation: bool,
    ) -> StructuredCellArray {
        let mut implicit_cell_array = StructuredCellArray::new();
        implicit_cell_array.set_data(extent, use_pixel_voxel_orientation);
        implicit_cell_array
    }

    /// VTK: `vtkStructuredData::GetCellTypes`.
    pub fn get_cell_types(
        extent: [i32; 6],
        use_pixel_voxel_orientation: bool,
    ) -> UnsignedCharConstantArray {
        let cell_type = Self::cell_type_for_extent(extent, use_pixel_voxel_orientation);
        let mut cell_types_array = UnsignedCharConstantArray::new();
        cell_types_array.construct_backend(cell_type as u8);
        cell_types_array.set_number_of_components(1);
        cell_types_array.set_number_of_tuples(Self::get_number_of_cells(extent));
        cell_types_array
    }

    /// VTK: `vtkStructuredData::GetCellTypesArray`.
    pub fn get_cell_types_array(
        extent: [i32; 6],
        use_pixel_voxel_orientation: bool,
    ) -> IntConstantArray {
        let result_unsigned_char = Self::get_cell_types(extent, use_pixel_voxel_orientation);
        let mut result_int = IntConstantArray::new();
        result_int.construct_backend(i32::from(result_unsigned_char.get_constant_value()));
        result_int.set_number_of_components(1);
        result_int.set_number_of_tuples(result_unsigned_char.get_number_of_tuples());
        result_int
    }

    /// VTK: `vtkStructuredData::GetPoints`.
    pub fn get_points(
        x_coordinates: &AnyArray,
        y_coordinates: &AnyArray,
        z_coordinates: &AnyArray,
        extent: [i32; 6],
        direction_matrix: [f64; 9],
    ) -> Points {
        let implicit_point_array = StructuredPointArray::create_structured_point_array(
            x_coordinates,
            y_coordinates,
            z_coordinates,
            extent,
            Self::get_data_description_from_extent(extent),
            direction_matrix,
        );
        let mut points = Points::new();
        points.set_data(&AnyArray::StructuredPoint(implicit_point_array));
        points
    }

    /// VTK: `vtkStructuredData::ComputePointId`.
    pub fn compute_point_id(dims: [i32; 3], ijk: [i32; 3]) -> VtkIdType {
        Self::get_linear_index(ijk[0], ijk[1], ijk[2], dims[0], dims[1])
    }

    /// VTK: `vtkStructuredData::ComputeCellId`.
    pub fn compute_cell_id(dims: [i32; 3], ijk: [i32; 3]) -> VtkIdType {
        Self::get_linear_index(
            ijk[0],
            ijk[1],
            ijk[2],
            (dims[0] - 1).max(1),
            (dims[1] - 1).max(1),
        )
    }

    /// VTK: `vtkStructuredData::ComputePointIdForExtent`.
    pub fn compute_point_id_for_extent(extent: [i32; 6], ijk: [i32; 3]) -> VtkIdType {
        let dims = Self::get_dimensions_from_extent(extent);
        let lijk = Self::get_local_structured_coordinates(ijk, extent);
        Self::compute_point_id(dims, lijk)
    }

    /// VTK: `vtkStructuredData::ComputeCellIdForExtent`.
    pub fn compute_cell_id_for_extent(extent: [i32; 6], ijk: [i32; 3]) -> VtkIdType {
        let dims = Self::get_dimensions_from_extent(extent);
        let lijk = Self::get_local_structured_coordinates(ijk, extent);
        Self::compute_cell_id(dims, lijk)
    }

    /// VTK: `vtkStructuredData::ComputeCellStructuredCoords`.
    pub fn compute_cell_structured_coords(cell_id: VtkIdType, dims: [i32; 3]) -> [i32; 3] {
        Self::get_structured_coordinates(cell_id, dims[0] - 1, dims[1] - 1)
    }

    /// VTK: `vtkStructuredData::ComputeCellStructuredCoordsForExtent`.
    pub fn compute_cell_structured_coords_for_extent(
        cell_id: VtkIdType,
        ext: [i32; 6],
    ) -> [i32; 3] {
        let dims = Self::get_dimensions_from_extent(ext);
        let lijk = Self::compute_cell_structured_coords(cell_id, dims);
        Self::get_global_structured_coordinates(lijk, ext)
    }

    /// VTK: `vtkStructuredData::ComputeCellStructuredMinMaxCoords`.
    pub fn compute_cell_structured_min_max_coords(
        cell_id: VtkIdType,
        dims: [i32; 3],
        data_description: i32,
    ) -> ([i32; 3], [i32; 3]) {
        match data_description {
            VTK_STRUCTURED_EMPTY | VTK_STRUCTURED_SINGLE_POINT => ([0, 0, 0], [0, 0, 0]),
            VTK_STRUCTURED_X_LINE => ([cell_id as i32, 0, 0], [cell_id as i32 + 1, 0, 0]),
            VTK_STRUCTURED_Y_LINE => ([0, cell_id as i32, 0], [0, cell_id as i32 + 1, 0]),
            VTK_STRUCTURED_Z_LINE => ([0, 0, cell_id as i32], [0, 0, cell_id as i32 + 1]),
            VTK_STRUCTURED_XY_PLANE => {
                let i = (cell_id % VtkIdType::from(dims[0] - 1)) as i32;
                let j = (cell_id / VtkIdType::from(dims[0] - 1)) as i32;
                ([i, j, 0], [i + 1, j + 1, 0])
            }
            VTK_STRUCTURED_YZ_PLANE => {
                let j = (cell_id % VtkIdType::from(dims[1] - 1)) as i32;
                let k = (cell_id / VtkIdType::from(dims[1] - 1)) as i32;
                ([0, j, k], [0, j + 1, k + 1])
            }
            VTK_STRUCTURED_XZ_PLANE => {
                let i = (cell_id % VtkIdType::from(dims[0] - 1)) as i32;
                let k = (cell_id / VtkIdType::from(dims[0] - 1)) as i32;
                ([i, 0, k], [i + 1, 0, k + 1])
            }
            VTK_STRUCTURED_XYZ_GRID => {
                let i = (cell_id % VtkIdType::from(dims[0] - 1)) as i32;
                let j = ((cell_id / VtkIdType::from(dims[0] - 1)) % VtkIdType::from(dims[1] - 1))
                    as i32;
                let k = (cell_id / (VtkIdType::from(dims[0] - 1) * VtkIdType::from(dims[1] - 1)))
                    as i32;
                ([i, j, k], [i + 1, j + 1, k + 1])
            }
            _ => ([0, 0, 0], [0, 0, 0]),
        }
    }

    /// VTK: `vtkStructuredData::ComputePointStructuredCoords`.
    pub fn compute_point_structured_coords(point_id: VtkIdType, dims: [i32; 3]) -> [i32; 3] {
        Self::get_structured_coordinates(point_id, dims[0], dims[1])
    }

    /// VTK: `vtkStructuredData::ComputePointStructuredCoordsForExtent`.
    pub fn compute_point_structured_coords_for_extent(
        point_id: VtkIdType,
        ext: [i32; 6],
    ) -> [i32; 3] {
        let dims = Self::get_dimensions_from_extent(ext);
        let lijk = Self::compute_point_structured_coords(point_id, dims);
        Self::get_global_structured_coordinates(lijk, ext)
    }

    /// VTK: `vtkStructuredData::GetCellPoints`.
    pub fn get_cell_points(
        cell_id: VtkIdType,
        point_ids: &mut IdList,
        data_description: i32,
        dims: [i32; 3],
    ) {
        point_ids.reset();
        for point_id in Self::cell_point_ids(cell_id, data_description, dims, true) {
            point_ids.insert_next_id(point_id);
        }
    }

    /// VTK: `vtkStructuredData::GetPointCells`.
    pub fn get_point_cells(point_id: VtkIdType, cell_ids: &mut IdList, dimensions: [i32; 3]) {
        cell_ids.reset();
        if point_id < 0 || dimensions[0] <= 0 || dimensions[1] <= 0 {
            return;
        }

        let cell_dimensions = dimensions.map(|dimension| {
            let cell_dimension = dimension - 1;
            if cell_dimension == 0 {
                1
            } else {
                cell_dimension
            }
        });
        let point_location = [
            (point_id % VtkIdType::from(dimensions[0])) as i32,
            ((point_id / VtkIdType::from(dimensions[0])) % VtkIdType::from(dimensions[1])) as i32,
            (point_id / (VtkIdType::from(dimensions[0]) * VtkIdType::from(dimensions[1]))) as i32,
        ];

        for offset in point_cell_offsets() {
            let mut cell_location = [0; 3];
            let mut valid = true;
            for axis in 0..3 {
                cell_location[axis] = point_location[axis] + offset[axis];
                if cell_location[axis] < 0 || cell_location[axis] >= cell_dimensions[axis] {
                    valid = false;
                    break;
                }
            }

            if valid {
                cell_ids.insert_next_id(Self::compute_cell_id(dimensions, cell_location));
            }
        }
    }

    /// VTK: `vtkStructuredData::GetCellNeighbors`.
    pub fn get_cell_neighbors(
        cell_id: VtkIdType,
        point_ids: &IdList,
        cell_ids: &mut IdList,
        dimensions: [i32; 3],
    ) {
        cell_ids.reset();
        if point_ids.get_number_of_ids() == 0 || dimensions[0] <= 0 || dimensions[1] <= 0 {
            return;
        }

        let id = point_ids.get_id(0);
        let seed_loc = [
            (id % VtkIdType::from(dimensions[0])) as i32,
            ((id / VtkIdType::from(dimensions[0])) % VtkIdType::from(dimensions[1])) as i32,
            (id / (VtkIdType::from(dimensions[0]) * VtkIdType::from(dimensions[1]))) as i32,
        ];

        Self::get_cell_neighbors_with_seed(cell_id, point_ids, cell_ids, dimensions, seed_loc);
    }

    /// VTK: `vtkStructuredData::GetCellNeighbors` with seed location.
    pub fn get_cell_neighbors_with_seed(
        cell_id: VtkIdType,
        point_ids: &IdList,
        cell_ids: &mut IdList,
        dimensions: [i32; 3],
        seed_loc: [i32; 3],
    ) {
        cell_ids.reset();
        if point_ids.get_number_of_ids() == 0 || dimensions[0] <= 0 || dimensions[1] <= 0 {
            return;
        }

        let mut offsets = neighbor_offsets();
        let id0 = VtkIdType::from(seed_loc[0])
            + VtkIdType::from(seed_loc[1]) * VtkIdType::from(dimensions[0])
            + VtkIdType::from(seed_loc[2])
                * VtkIdType::from(dimensions[0])
                * VtkIdType::from(dimensions[1]);

        for idx in 0..point_ids.get_number_of_ids() {
            let id = point_ids.get_id(idx);
            trim_neighbor_offsets_from_point_id(id, id0, dimensions, &mut offsets);
        }

        insert_neighbor_cells(cell_id, cell_ids, dimensions, seed_loc, offsets);
    }

    /// VTK: `vtkStructuredData::IsPointVisible`.
    pub fn is_point_visible(point_id: VtkIdType, ghosts: Option<&AnyArray>) -> bool {
        if point_id < 0 {
            return false;
        }
        let Some(ghosts) = ghosts else {
            return true;
        };
        ghost_value(ghosts, point_id).is_some_and(|value| value & HIDDENPOINT == 0)
    }

    /// VTK: `vtkStructuredData::IsCellVisible`.
    pub fn is_cell_visible(
        cell_id: VtkIdType,
        dimensions: [i32; 3],
        data_description: i32,
        cell_ghost_array: Option<&AnyArray>,
        point_ghost_array: Option<&AnyArray>,
    ) -> bool {
        const MASKED_CELL_VALUE: u8 = HIDDENCELL | REFINEDCELL;

        if cell_id < 0 || data_description == VTK_STRUCTURED_EMPTY {
            return false;
        }

        if cell_ghost_array
            .and_then(|ghosts| ghost_value(ghosts, cell_id))
            .is_some_and(|value| value & MASKED_CELL_VALUE != 0)
        {
            return false;
        }

        let Some(point_ghost_array) = point_ghost_array else {
            return true;
        };

        Self::cell_point_ids(cell_id, data_description, dimensions, false)
            .into_iter()
            .all(|point_id| Self::is_point_visible(point_id, Some(point_ghost_array)))
    }

    pub(crate) fn cell_point_ids_for_extent(
        cell_id: VtkIdType,
        extent: [i32; 6],
        use_pixel_voxel_orientation: bool,
    ) -> Vec<VtkIdType> {
        let dims = Self::get_dimensions_from_extent(extent);
        let data_description = Self::get_data_description(dims);
        Self::cell_point_ids(cell_id, data_description, dims, use_pixel_voxel_orientation)
    }

    pub(crate) fn cell_type_for_extent(
        extent: [i32; 6],
        use_pixel_voxel_orientation: bool,
    ) -> CellType {
        let data_description = Self::get_data_description_from_extent(extent);
        match Self::get_data_dimension(data_description) {
            3 => {
                if use_pixel_voxel_orientation {
                    CellType::Voxel
                } else {
                    CellType::Hexahedron
                }
            }
            2 => {
                if use_pixel_voxel_orientation {
                    CellType::Pixel
                } else {
                    CellType::Quad
                }
            }
            1 => CellType::Line,
            0 if data_description == VTK_STRUCTURED_SINGLE_POINT => CellType::Vertex,
            _ => CellType::Empty,
        }
    }

    pub(crate) fn cell_structured_min_max_coords_for_extent(
        cell_id: VtkIdType,
        extent: [i32; 6],
    ) -> Option<([i32; 3], [i32; 3])> {
        if cell_id < 0 || cell_id >= Self::get_number_of_cells(extent) {
            return None;
        }
        let dims = Self::get_dimensions_from_extent(extent);
        let data_description = Self::get_data_description(dims);
        if data_description == VTK_STRUCTURED_EMPTY {
            return None;
        }
        let (ijk_min, ijk_max) =
            Self::compute_cell_structured_min_max_coords(cell_id, dims, data_description);
        let origin = [extent[0], extent[2], extent[4]];
        Some((
            [
                origin[0] + ijk_min[0],
                origin[1] + ijk_min[1],
                origin[2] + ijk_min[2],
            ],
            [
                origin[0] + ijk_max[0],
                origin[1] + ijk_max[1],
                origin[2] + ijk_max[2],
            ],
        ))
    }

    pub(crate) fn cell_size_for_extent(cell_id: VtkIdType, extent: [i32; 6]) -> VtkIdType {
        Self::cell_structured_min_max_coords_for_extent(cell_id, extent).map_or(0, |(min, max)| {
            VtkIdType::from(max[0] - min[0] + 1)
                * VtkIdType::from(max[1] - min[1] + 1)
                * VtkIdType::from(max[2] - min[2] + 1)
        })
    }

    fn cell_point_ids(
        cell_id: VtkIdType,
        data_description: i32,
        dims: [i32; 3],
        use_pixel_voxel_orientation: bool,
    ) -> Vec<VtkIdType> {
        if data_description == VTK_STRUCTURED_EMPTY {
            return Vec::new();
        }
        let ijk = Self::cell_origin(cell_id, data_description, dims);
        let shifts = shift_lut(data_description, use_pixel_voxel_orientation);
        let size = cell_size(data_description);
        let mut point_ids = Vec::with_capacity(size);
        for comp in 0..size {
            point_ids.push(Self::compute_point_id(
                dims,
                [
                    ijk[0] + shifts[0][comp],
                    ijk[1] + shifts[1][comp],
                    ijk[2] + shifts[2][comp],
                ],
            ));
        }
        point_ids
    }

    fn cell_origin(cell_id: VtkIdType, data_description: i32, dims: [i32; 3]) -> [i32; 3] {
        match data_description {
            VTK_STRUCTURED_EMPTY | VTK_STRUCTURED_SINGLE_POINT => [0, 0, 0],
            VTK_STRUCTURED_X_LINE => [cell_id as i32, 0, 0],
            VTK_STRUCTURED_Y_LINE => [0, cell_id as i32, 0],
            VTK_STRUCTURED_Z_LINE => [0, 0, cell_id as i32],
            VTK_STRUCTURED_XY_PLANE => [
                (cell_id % VtkIdType::from(dims[0] - 1)) as i32,
                (cell_id / VtkIdType::from(dims[0] - 1)) as i32,
                0,
            ],
            VTK_STRUCTURED_YZ_PLANE => [
                0,
                (cell_id % VtkIdType::from(dims[1] - 1)) as i32,
                (cell_id / VtkIdType::from(dims[1] - 1)) as i32,
            ],
            VTK_STRUCTURED_XZ_PLANE => [
                (cell_id % VtkIdType::from(dims[0] - 1)) as i32,
                0,
                (cell_id / VtkIdType::from(dims[0] - 1)) as i32,
            ],
            VTK_STRUCTURED_XYZ_GRID => [
                (cell_id % VtkIdType::from(dims[0] - 1)) as i32,
                ((cell_id / VtkIdType::from(dims[0] - 1)) % VtkIdType::from(dims[1] - 1)) as i32,
                (cell_id / (VtkIdType::from(dims[0] - 1) * VtkIdType::from(dims[1] - 1))) as i32,
            ],
            _ => [0, 0, 0],
        }
    }

    fn get_linear_index(i: i32, j: i32, k: i32, n1: i32, n2: i32) -> VtkIdType {
        (VtkIdType::from(k) * VtkIdType::from(n2) + VtkIdType::from(j)) * VtkIdType::from(n1)
            + VtkIdType::from(i)
    }

    fn get_structured_coordinates(idx: VtkIdType, n1: i32, n2: i32) -> [i32; 3] {
        let n12 = VtkIdType::from(n1) * VtkIdType::from(n2);
        let k = idx / n12;
        let j = (idx - k * n12) / VtkIdType::from(n1);
        let i = idx - k * n12 - j * VtkIdType::from(n1);
        [i as i32, j as i32, k as i32]
    }
}

fn cell_size(data_description: i32) -> usize {
    match data_description {
        VTK_STRUCTURED_XYZ_GRID => 8,
        VTK_STRUCTURED_XY_PLANE | VTK_STRUCTURED_YZ_PLANE | VTK_STRUCTURED_XZ_PLANE => 4,
        VTK_STRUCTURED_X_LINE | VTK_STRUCTURED_Y_LINE | VTK_STRUCTURED_Z_LINE => 2,
        VTK_STRUCTURED_SINGLE_POINT => 1,
        _ => 0,
    }
}

fn shift_lut(data_description: i32, use_pixel_voxel_orientation: bool) -> [[i32; 8]; 3] {
    let zero = [0, 0, 0, 0, 0, 0, 0, 0];
    let z = [0, 0, 0, 0, 1, 1, 1, 1];
    let pixel_x = [0, 1, 0, 1, 0, 1, 0, 1];
    let pixel_y = [0, 0, 1, 1, 0, 0, 1, 1];
    let quad_x = [0, 1, 1, 0, 0, 1, 1, 0];
    let quad_y = [0, 0, 1, 1, 0, 0, 1, 1];

    match (data_description, use_pixel_voxel_orientation) {
        (VTK_STRUCTURED_X_LINE, true) => [pixel_x, zero, zero],
        (VTK_STRUCTURED_Y_LINE, true) => [zero, pixel_x, zero],
        (VTK_STRUCTURED_Z_LINE, true) => [zero, zero, pixel_x],
        (VTK_STRUCTURED_XY_PLANE, true) => [pixel_x, pixel_y, zero],
        (VTK_STRUCTURED_YZ_PLANE, true) => [zero, pixel_x, pixel_y],
        (VTK_STRUCTURED_XZ_PLANE, true) => [pixel_x, zero, pixel_y],
        (VTK_STRUCTURED_XYZ_GRID, true) => [pixel_x, pixel_y, z],
        (VTK_STRUCTURED_X_LINE, false) => [quad_x, zero, zero],
        (VTK_STRUCTURED_Y_LINE, false) => [zero, quad_x, zero],
        (VTK_STRUCTURED_Z_LINE, false) => [zero, zero, quad_x],
        (VTK_STRUCTURED_XY_PLANE, false) => [quad_x, quad_y, zero],
        (VTK_STRUCTURED_YZ_PLANE, false) => [zero, quad_x, quad_y],
        (VTK_STRUCTURED_XZ_PLANE, false) => [quad_x, zero, quad_y],
        (VTK_STRUCTURED_XYZ_GRID, false) => [quad_x, quad_y, z],
        _ => [zero, zero, zero],
    }
}

fn neighbor_offsets() -> [[i32; 3]; 8] {
    [
        [-1, -1, -1],
        [0, -1, -1],
        [-1, 0, -1],
        [0, 0, -1],
        [-1, -1, 0],
        [0, -1, 0],
        [-1, 0, 0],
        [0, 0, 0],
    ]
}

fn point_cell_offsets() -> [[i32; 3]; 8] {
    [
        [-1, 0, 0],
        [-1, -1, 0],
        [-1, -1, -1],
        [-1, 0, -1],
        [0, 0, 0],
        [0, -1, 0],
        [0, -1, -1],
        [0, 0, -1],
    ]
}

fn trim_neighbor_offsets_from_point_id(
    id: VtkIdType,
    seed_id: VtkIdType,
    dimensions: [i32; 3],
    offsets: &mut [[i32; 3]; 8],
) {
    let dim_i = VtkIdType::from(dimensions[0]);
    let dim_ij = dim_i * VtkIdType::from(dimensions[1]);

    if id - 1 == seed_id {
        for idx in [0, 2, 4, 6] {
            offsets[idx][0] = -10;
        }
    } else if id + 1 == seed_id {
        for idx in [1, 3, 5, 7] {
            offsets[idx][0] = -10;
        }
    } else if id - dim_i == seed_id {
        for idx in [0, 1, 4, 5] {
            offsets[idx][1] = -10;
        }
    } else if id + dim_i == seed_id {
        for idx in [2, 3, 6, 7] {
            offsets[idx][1] = -10;
        }
    } else if id - dim_ij == seed_id {
        for idx in [0, 1, 2, 3] {
            offsets[idx][2] = -10;
        }
    } else if id + dim_ij == seed_id {
        for idx in [4, 5, 6, 7] {
            offsets[idx][2] = -10;
        }
    }
}

fn insert_neighbor_cells(
    cell_id: VtkIdType,
    cell_ids: &mut IdList,
    dimensions: [i32; 3],
    seed_loc: [i32; 3],
    offsets: [[i32; 3]; 8],
) {
    let cell_dims = dimensions.map(|dimension| (dimension - 1).max(1));

    for offset in offsets {
        let mut cell_loc = [0; 3];
        let mut valid = true;
        for axis in 0..3 {
            if offset[axis] == -10 {
                valid = false;
                break;
            }

            cell_loc[axis] = seed_loc[axis] + offset[axis];
            if cell_loc[axis] < 0 || cell_loc[axis] >= cell_dims[axis] {
                valid = false;
                break;
            }
        }

        if valid {
            let id = StructuredData::compute_cell_id(dimensions, cell_loc);
            if id != cell_id {
                cell_ids.insert_next_id(id);
            }
        }
    }
}

fn ghost_value(ghosts: &AnyArray, tuple_id: VtkIdType) -> Option<u8> {
    if tuple_id < 0 {
        return None;
    }
    let tuple = ghosts
        .numeric_tuple_as_f64_checked(tuple_id as usize)
        .ok()?;
    tuple.first().map(|value| *value as u8)
}
