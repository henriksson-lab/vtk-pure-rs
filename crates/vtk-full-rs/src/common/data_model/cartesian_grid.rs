use crate::common::core::{AnyArray, IdList, VtkDataType, VtkIdType};

use super::{
    CellType, DataObjectType, DataSet, DataSetAttributes, StructuredCellArray, StructuredData,
    HIDDENCELL, HIDDENPOINT, VTK_STRUCTURED_EMPTY, VTK_STRUCTURED_UNCHANGED,
};

/// VTK origin: `VTK_3D_EXTENT`.
pub const VTK_3D_EXTENT: i32 = 1;

#[derive(Debug, Clone, PartialEq)]
struct CartesianGridStorage {
    data_set: DataSet,
    data_description: i32,
    dimensions: [i32; 3],
    extent: [i32; 6],
    point: [f64; 3],
    structured_cells: Option<StructuredCellArray>,
    structured_cell_type: i32,
    modified_time: u64,
}

/// Abstract common base for `vtkImageData` and `vtkRectilinearGrid`.
///
/// VTK origin: `VTK/Common/DataModel/vtkCartesianGrid.h` and
/// `VTK/Common/DataModel/vtkCartesianGrid.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct CartesianGrid {
    storage: CartesianGridStorage,
}

impl CartesianGrid {
    /// VTK: `vtkCartesianGrid::vtkCartesianGrid`.
    #[allow(dead_code)]
    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            storage: CartesianGridStorage {
                data_set: DataSet::with_type(object_type),
                data_description: VTK_STRUCTURED_EMPTY,
                dimensions: [0, 0, 0],
                extent: [0, -1, 0, -1, 0, -1],
                point: [0.0; 3],
                structured_cells: None,
                structured_cell_type: CellType::Empty as i32,
                modified_time: 0,
            },
        }
    }

    #[allow(dead_code)]
    pub(crate) fn data_set(&self) -> &DataSet {
        &self.storage.data_set
    }

    #[allow(dead_code)]
    pub(crate) fn data_set_mut(&mut self) -> &mut DataSet {
        self.modified();
        &mut self.storage.data_set
    }

    /// VTK: `vtkCartesianGrid::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkCartesianGrid\n  DataDescription: {}\n  Dimensions: ({}, {}, {})\n  Extent: ({}, {}, {}, {}, {}, {})",
            self.storage.data_description,
            self.storage.dimensions[0],
            self.storage.dimensions[1],
            self.storage.dimensions[2],
            self.storage.extent[0],
            self.storage.extent[1],
            self.storage.extent[2],
            self.storage.extent[3],
            self.storage.extent[4],
            self.storage.extent[5],
        )
    }

    /// VTK: `vtkCartesianGrid::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage.data_set.shallow_copy(&source.storage.data_set);
        self.copy_structure(source);
    }

    /// VTK: `vtkCartesianGrid::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage.data_set.deep_copy(&source.storage.data_set);
        self.copy_structure(source);
    }

    /// VTK: `vtkCartesianGrid::CopyStructure`.
    pub fn copy_structure(&mut self, source: &Self) {
        self.set_extent(source.storage.extent);
    }

    /// VTK: `vtkCartesianGrid::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        51
    }

    /// VTK: `vtkCartesianGrid::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        StructuredData::get_number_of_points(self.storage.extent)
    }

    /// VTK: `vtkCartesianGrid::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        StructuredData::get_number_of_cells(self.storage.extent)
    }

    /// VTK: `vtkCartesianGrid::GetCellType`.
    pub fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        if !self.is_cell_visible(cell_id) {
            return CellType::Empty as i32;
        }
        self.storage.structured_cell_type
    }

    /// VTK: `vtkCartesianGrid::GetCellSize`.
    pub fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType {
        if !self.is_cell_visible(cell_id) {
            return 0;
        }
        self.structured_cells().get_cell_size(cell_id)
    }

    /// VTK: `vtkCartesianGrid::GetCellPoints(vtkIdType, vtkIdList*)`.
    pub fn get_cell_points(&self, cell_id: VtkIdType, pt_ids: &mut IdList) {
        self.structured_cells()
            .get_cell_at_id_into_id_list(cell_id, pt_ids);
    }

    /// VTK: `vtkCartesianGrid::GetCellPoints(vtkIdType, vtkIdType&, vtkIdType const*&, vtkIdList*)`.
    pub fn get_cell_points_with_temp(
        &self,
        cell_id: VtkIdType,
        pt_ids: &mut IdList,
    ) -> Vec<VtkIdType> {
        self.structured_cells()
            .get_cell_at_id_with_temp(cell_id, pt_ids)
    }

    /// VTK: `vtkCartesianGrid::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        8
    }

    /// VTK: `vtkCartesianGrid::GetPointCells`.
    pub fn get_point_cells(&self, point_id: VtkIdType, cell_ids: &mut IdList) {
        StructuredData::get_point_cells(point_id, cell_ids, self.get_dimensions());
    }

    /// VTK: `vtkCartesianGrid::GetCellNeighbors`.
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

    /// VTK: `vtkCartesianGrid::GetCellNeighbors` with seed location.
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

    /// VTK: `vtkCartesianGrid::HasAnyBlankPoints`.
    pub fn has_any_blank_points(&self) -> bool {
        self.storage
            .data_set
            .get_point_data()
            .has_any_ghost_bit_set(i32::from(HIDDENPOINT))
    }

    /// VTK: `vtkCartesianGrid::HasAnyBlankCells`.
    pub fn has_any_blank_cells(&self) -> bool {
        self.storage
            .data_set
            .get_cell_data()
            .has_any_ghost_bit_set(i32::from(HIDDENCELL))
            || self.has_any_blank_points()
    }

    /// VTK: `vtkCartesianGrid::Initialize`.
    pub fn initialize(&mut self) {
        self.storage.data_set.initialize();
        self.set_dimensions([0, 0, 0]);
    }

    /// VTK: `vtkCartesianGrid::GetDataDimension`.
    pub fn get_data_dimension(&self) -> i32 {
        StructuredData::get_data_dimension(self.storage.data_description)
    }

    /// VTK: `vtkCartesianGrid::GetMaxSpatialDimension`.
    pub fn get_max_spatial_dimension(&self) -> i32 {
        self.get_data_dimension()
    }

    /// VTK: `vtkCartesianGrid::GetMinSpatialDimension`.
    pub fn get_min_spatial_dimension(&self) -> i32 {
        self.get_data_dimension()
    }

    /// VTK: `vtkCartesianGrid::GetCells`.
    pub fn get_cells(&self) -> Option<&StructuredCellArray> {
        self.storage.structured_cells.as_ref()
    }

    /// VTK: `vtkCartesianGrid::GetCellDims`.
    pub fn get_cell_dims(&self) -> [i32; 3] {
        self.storage.dimensions.map(|dim| (dim - 1).max(1))
    }

    /// VTK: `vtkCartesianGrid::IsPointVisible`.
    pub fn is_point_visible(&self, point_id: VtkIdType) -> u8 {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return 0;
        }
        u8::from(StructuredData::is_point_visible(
            point_id,
            self.storage.data_set.get_point_data().get_ghost_array(),
        ))
    }

    /// VTK: `vtkCartesianGrid::BlankPoint`.
    pub fn blank_point(&mut self, point_id: VtkIdType) {
        let number_of_points = self.get_number_of_points();
        if point_id < 0 || point_id >= number_of_points {
            return;
        }
        self.storage
            .data_set
            .get_point_data_mut()
            .allocate_ghost_array(number_of_points);
        self.storage
            .data_set
            .get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, true);
        self.modified();
    }

    /// VTK: `vtkCartesianGrid::BlankPoint(int, int, int)`.
    pub fn blank_point_ijk(&mut self, i: i32, j: i32, k: i32) {
        let point_id = StructuredData::compute_point_id(self.get_dimensions(), [i, j, k]);
        self.blank_point(point_id);
    }

    /// VTK: `vtkCartesianGrid::UnBlankPoint`.
    pub fn un_blank_point(&mut self, point_id: VtkIdType) {
        if point_id < 0 || point_id >= self.get_number_of_points() {
            return;
        }
        if self
            .storage
            .data_set
            .get_point_data_mut()
            .set_ghost_bit(point_id, HIDDENPOINT, false)
        {
            self.modified();
        }
    }

    /// VTK: `vtkCartesianGrid::UnBlankPoint(int, int, int)`.
    pub fn un_blank_point_ijk(&mut self, i: i32, j: i32, k: i32) {
        let point_id = StructuredData::compute_point_id(self.get_dimensions(), [i, j, k]);
        self.un_blank_point(point_id);
    }

    /// VTK: `vtkCartesianGrid::BlankCell`.
    pub fn blank_cell(&mut self, cell_id: VtkIdType) {
        let number_of_cells = self.get_number_of_cells();
        if cell_id < 0 || cell_id >= number_of_cells {
            return;
        }
        self.storage
            .data_set
            .get_cell_data_mut()
            .allocate_ghost_array(number_of_cells);
        self.storage
            .data_set
            .get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, true);
        self.modified();
    }

    /// VTK: `vtkCartesianGrid::BlankCell(int, int, int)`.
    pub fn blank_cell_ijk(&mut self, i: i32, j: i32, k: i32) {
        let cell_id = StructuredData::compute_cell_id(self.get_dimensions(), [i, j, k]);
        self.blank_cell(cell_id);
    }

    /// VTK: `vtkCartesianGrid::UnBlankCell`.
    pub fn un_blank_cell(&mut self, cell_id: VtkIdType) {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return;
        }
        if self
            .storage
            .data_set
            .get_cell_data_mut()
            .set_ghost_bit(cell_id, HIDDENCELL, false)
        {
            self.modified();
        }
    }

    /// VTK: `vtkCartesianGrid::UnBlankCell(int, int, int)`.
    pub fn un_blank_cell_ijk(&mut self, i: i32, j: i32, k: i32) {
        let cell_id = StructuredData::compute_cell_id(self.get_dimensions(), [i, j, k]);
        self.un_blank_cell(cell_id);
    }

    /// VTK: `vtkCartesianGrid::IsCellVisible`.
    pub fn is_cell_visible(&self, cell_id: VtkIdType) -> bool {
        if cell_id < 0 || cell_id >= self.get_number_of_cells() {
            return false;
        }
        StructuredData::is_cell_visible(
            cell_id,
            self.get_dimensions(),
            self.storage.data_description,
            self.storage.data_set.get_cell_data().get_ghost_array(),
            self.storage.data_set.get_point_data().get_ghost_array(),
        )
    }

    /// VTK: `vtkCartesianGrid::GetDataDescription`.
    pub fn get_data_description(&self) -> i32 {
        self.storage.data_description
    }

    /// VTK: `vtkCartesianGrid::SetDimensions`.
    pub fn set_dimensions(&mut self, dimensions: [i32; 3]) {
        self.set_extent([
            0,
            dimensions[0] - 1,
            0,
            dimensions[1] - 1,
            0,
            dimensions[2] - 1,
        ]);
    }

    /// VTK: `vtkCartesianGrid::GetDimensions`.
    pub fn get_dimensions(&self) -> [i32; 3] {
        StructuredData::get_dimensions_from_extent(self.storage.extent)
    }

    /// VTK: `vtkCartesianGrid::GetDimensions(int[3])`.
    pub fn get_dimensions_into(&self, dims: &mut [i32; 3]) {
        *dims = self.get_dimensions();
    }

    /// VTK: `vtkCartesianGrid::GetExtentType`.
    pub fn get_extent_type(&self) -> i32 {
        VTK_3D_EXTENT
    }

    /// VTK: `vtkCartesianGrid::SetExtent`.
    pub fn set_extent(&mut self, extent: [i32; 6]) {
        let description = StructuredData::set_extent(extent, &mut self.storage.extent);
        if description < 0 || description == VTK_STRUCTURED_UNCHANGED {
            return;
        }

        self.storage.dimensions = StructuredData::get_dimensions_from_extent(extent);
        self.storage.data_description = description;
        self.build_implicit_structures();
        self.modified();
    }

    /// VTK: `vtkCartesianGrid::SetExtent(int, int, int, int, int, int)`.
    pub fn set_extent_values(
        &mut self,
        x_min: i32,
        x_max: i32,
        y_min: i32,
        y_max: i32,
        z_min: i32,
        z_max: i32,
    ) {
        self.set_extent([x_min, x_max, y_min, y_max, z_min, z_max]);
    }

    /// VTK: `vtkCartesianGrid::GetExtent`.
    pub fn get_extent(&self) -> [i32; 6] {
        self.storage.extent
    }

    /// VTK: `vtkCartesianGrid::GetScalarType`.
    pub fn get_scalar_type(&self) -> i32 {
        self.storage
            .data_set
            .get_point_data()
            .get_field_data_scalars()
            .map_or(VtkDataType::VTK_DOUBLE, |scalars| {
                scalars.get_data_type().id()
            })
    }

    /// VTK: `vtkCartesianGrid::SetScalarType`.
    pub fn set_scalar_type(&mut self, scalar_type: VtkDataType) {
        if let Some((name, components, tuples)) = self
            .storage
            .data_set
            .get_point_data()
            .get_field_data_scalars()
            .map(|scalars| {
                (
                    scalars.get_name().to_string(),
                    scalars.get_number_of_components(),
                    scalars.get_number_of_tuples(),
                )
            })
        {
            if self.get_scalar_type() == scalar_type.id() {
                return;
            }
            let mut replacement =
                AnyArray::create_array(scalar_type).expect("supported scalar data type");
            replacement.set_name(&name);
            replacement.set_number_of_components(
                i32::try_from(components).expect("component count must fit int"),
            );
            replacement
                .set_number_of_tuples(VtkIdType::try_from(tuples).expect("tuple count must fit"));
            self.storage
                .data_set
                .get_point_data_mut()
                .set_scalars(Some(replacement));
        } else {
            let mut scalars =
                AnyArray::create_array(scalar_type).expect("supported scalar data type");
            scalars.set_name("ImageScalars");
            scalars.set_number_of_components(1);
            scalars.set_number_of_tuples(self.get_number_of_points());
            self.storage
                .data_set
                .get_point_data_mut()
                .set_scalars(Some(scalars));
        }
    }

    /// VTK: `vtkCartesianGrid::GetNumberOfScalarComponents`.
    pub fn get_number_of_scalar_components(&self) -> i32 {
        self.storage
            .data_set
            .get_point_data()
            .get_field_data_scalars()
            .map_or(1, |scalars| scalars.get_number_of_components() as i32)
    }

    /// VTK: `vtkCartesianGrid::GetScalarTypeAsString`.
    pub fn get_scalar_type_as_string(&self) -> &'static str {
        VtkDataType::from_id(self.get_scalar_type()).map_or("Undefined", VtkDataType::vtk_name)
    }

    /// VTK: `vtkCartesianGrid::GetTupleIndex`.
    pub fn get_tuple_index(&self, array: Option<&AnyArray>, coordinates: [i32; 3]) -> VtkIdType {
        let Some(array) = array else {
            return -1;
        };
        let extent = self.storage.extent;
        for axis in 0..3 {
            if coordinates[axis] < extent[axis * 2] || coordinates[axis] > extent[axis * 2 + 1] {
                return -1;
            }
        }

        let incs = [
            1,
            VtkIdType::from(extent[1] - extent[0] + 1),
            VtkIdType::from((extent[1] - extent[0] + 1) * (extent[3] - extent[2] + 1)),
        ];
        let idx = VtkIdType::from(coordinates[0] - extent[0]) * incs[0]
            + VtkIdType::from(coordinates[1] - extent[2]) * incs[1]
            + VtkIdType::from(coordinates[2] - extent[4]) * incs[2];

        if idx < 0 || idx > array.get_number_of_values() - 1 {
            -1
        } else {
            idx
        }
    }

    /// VTK: `vtkCartesianGrid::GetTupleIndex(vtkDataArray*, int, int, int)`.
    pub fn get_tuple_index_values(
        &self,
        array: Option<&AnyArray>,
        x: i32,
        y: i32,
        z: i32,
    ) -> VtkIdType {
        self.get_tuple_index(array, [x, y, z])
    }

    /// VTK: `vtkCartesianGrid::GetTupleIndexForExtent`.
    pub fn get_tuple_index_for_extent(
        &self,
        array: Option<&AnyArray>,
        extent: [i32; 6],
    ) -> VtkIdType {
        self.get_tuple_index(array, [extent[0], extent[2], extent[4]])
    }

    /// VTK: `vtkCartesianGrid::GetValueIndex`.
    pub fn get_value_index(&self, array: Option<&AnyArray>, coordinates: [i32; 3]) -> VtkIdType {
        let tuple_index = self.get_tuple_index(array, coordinates);
        match (tuple_index, array) {
            (idx, Some(array)) if idx >= 0 => {
                idx * VtkIdType::from(array.get_number_of_components())
            }
            _ => -1,
        }
    }

    /// VTK: `vtkCartesianGrid::GetValueIndex(vtkDataArray*, int, int, int)`.
    pub fn get_value_index_values(
        &self,
        array: Option<&AnyArray>,
        x: i32,
        y: i32,
        z: i32,
    ) -> VtkIdType {
        self.get_value_index(array, [x, y, z])
    }

    /// VTK: `vtkCartesianGrid::GetValueIndexForExtent`.
    pub fn get_value_index_for_extent(
        &self,
        array: Option<&AnyArray>,
        extent: [i32; 6],
    ) -> VtkIdType {
        self.get_value_index(array, [extent[0], extent[2], extent[4]])
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkCartesianGrid"
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.storage.modified_time = self.storage.modified_time.saturating_add(1);
        self.storage.data_set.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.data_set.get_m_time())
    }

    #[allow(dead_code)]
    pub(crate) fn get_point_data(&self) -> &DataSetAttributes {
        self.storage.data_set.get_point_data()
    }

    fn build_implicit_structures(&mut self) {
        self.build_cells();
        self.build_cell_types();
    }

    fn build_cells(&mut self) {
        let mut cells = StructuredCellArray::new();
        cells.set_data(self.storage.extent, true);
        self.storage.structured_cells = Some(cells);
    }

    fn build_cell_types(&mut self) {
        self.storage.structured_cell_type =
            StructuredData::cell_type_for_extent(self.storage.extent, true) as i32;
    }

    fn structured_cells(&self) -> &StructuredCellArray {
        self.storage
            .structured_cells
            .as_ref()
            .expect("vtkCartesianGrid structured cells are null; call set_extent first")
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
                StructuredData::cell_point_ids_for_extent(candidate_id, self.storage.extent, true);
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
