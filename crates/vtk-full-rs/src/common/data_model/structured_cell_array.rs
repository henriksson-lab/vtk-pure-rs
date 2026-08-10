use std::sync::Arc;

use crate::common::core::{IdList, VtkIdType, VtkMTimeType};

use super::{
    AbstractCellArray, AbstractCellArrayApi, AbstractCellArrayHandle, StructuredData,
    VTK_STRUCTURED_EMPTY, VTK_STRUCTURED_SINGLE_POINT, VTK_STRUCTURED_XYZ_GRID,
    VTK_STRUCTURED_XY_PLANE, VTK_STRUCTURED_XZ_PLANE, VTK_STRUCTURED_X_LINE,
    VTK_STRUCTURED_YZ_PLANE, VTK_STRUCTURED_Y_LINE, VTK_STRUCTURED_Z_LINE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct StructuredCellArrayStorage {
    extent: [i32; 6],
    dimensions: [i32; 3],
    data_description: i32,
    number_of_cells: VtkIdType,
    cell_size: VtkIdType,
    use_pixel_voxel_orientation: bool,
}

/// Implicit cell connectivity for structured datasets.
///
/// VTK origin: `VTK/Common/DataModel/vtkStructuredCellArray.h` and
/// `VTK/Common/DataModel/vtkStructuredCellArray.cxx`.
#[derive(Debug)]
pub struct StructuredCellArray {
    abstract_cell_array: AbstractCellArray,
    connectivity: Option<Arc<StructuredCellArrayStorage>>,
}

impl Clone for StructuredCellArray {
    fn clone(&self) -> Self {
        Self {
            abstract_cell_array: self.abstract_cell_array.clone(),
            connectivity: self.connectivity.as_ref().map(Arc::clone),
        }
    }
}

impl PartialEq for StructuredCellArray {
    fn eq(&self, other: &Self) -> bool {
        match (&self.connectivity, &other.connectivity) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right) || left == right,
            (None, None) => true,
            _ => false,
        }
    }
}

impl StructuredCellArray {
    /// VTK: `vtkStructuredCellArray::New`.
    pub fn new() -> Self {
        Self {
            abstract_cell_array: AbstractCellArray::with_class_name("vtkStructuredCellArray"),
            connectivity: None,
        }
    }

    /// VTK: `vtkStructuredCellArray::PrintSelf`.
    pub fn print_self(&self) -> String {
        match self.connectivity.as_deref() {
            Some(storage) => format!(
                "vtkStructuredCellArray {{ Extent: {:?}, Dimensions: {:?}, DataDescription: {}, NumberOfCells: {}, CellSize: {}, UsePixelVoxelOrientation: {} }}",
                storage.extent,
                storage.dimensions,
                storage.data_description,
                storage.number_of_cells,
                storage.cell_size,
                storage.use_pixel_voxel_orientation
            ),
            None => "vtkStructuredCellArray { Connectivity: (nullptr) }".to_string(),
        }
    }

    /// VTK: `vtkStructuredCellArray::Initialize`.
    pub fn initialize(&mut self) {
        if let Some(storage) = self.connectivity.as_ref() {
            self.connectivity = Some(Arc::new(empty_storage(storage.use_pixel_voxel_orientation)));
        }
        self.modified();
    }

    /// VTK: `vtkStructuredCellArray::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        self.storage().number_of_cells
    }

    /// VTK: `vtkStructuredCellArray::GetNumberOfOffsets`.
    pub fn get_number_of_offsets(&self) -> VtkIdType {
        self.storage().number_of_cells + 1
    }

    /// VTK: `vtkStructuredCellArray::GetOffset`.
    pub fn get_offset(&self, cell_id: VtkIdType) -> VtkIdType {
        self.storage().cell_size * cell_id
    }

    /// VTK: `vtkStructuredCellArray::GetNumberOfConnectivityIds`.
    pub fn get_number_of_connectivity_ids(&self) -> VtkIdType {
        let storage = self.storage();
        storage.number_of_cells * storage.cell_size
    }

    /// VTK: `vtkStructuredCellArray::SetData`.
    pub fn set_data(&mut self, extent: [i32; 6], use_pixel_voxel_orientation: bool) {
        self.connectivity = Some(Arc::new(storage_from_extent(
            extent,
            use_pixel_voxel_orientation,
        )));
        self.modified();
    }

    /// VTK: `vtkStructuredCellArray::IsStorageShareable`.
    pub fn is_storage_shareable(&self) -> bool {
        false
    }

    /// VTK: `vtkStructuredCellArray::IsHomogeneous`.
    pub fn is_homogeneous(&self) -> VtkIdType {
        self.connectivity
            .as_ref()
            .map_or(0, |storage| storage.cell_size)
    }

    /// VTK: `vtkStructuredCellArray::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType const*&, vtkIdList*)`.
    pub fn get_cell_at_id_with_temp(
        &self,
        cell_id: VtkIdType,
        pt_ids: &mut IdList,
    ) -> Vec<VtkIdType> {
        self.get_cell_at_id_into_id_list(cell_id, pt_ids);
        id_list_values(pt_ids)
    }

    /// VTK: `vtkStructuredCellArray::GetCellAtId(vtkIdType, vtkIdList*)`.
    pub fn get_cell_at_id_into_id_list(&self, cell_id: VtkIdType, pt_ids: &mut IdList) {
        let values = self.cell_at_id(cell_id);
        write_id_list(pt_ids, &values);
    }

    /// VTK: `vtkStructuredCellArray::GetCellAtId(vtkIdType, vtkIdType&, vtkIdType*)`.
    pub fn get_cell_at_id_into_slice(&self, cell_id: VtkIdType, cell_points: &mut [VtkIdType]) {
        let values = self.cell_at_id(cell_id);
        assert!(
            cell_points.len() >= values.len(),
            "cell_points must have room for the structured cell"
        );
        cell_points[..values.len()].copy_from_slice(&values);
    }

    /// VTK: `vtkStructuredCellArray::GetCellAtId(int[3], vtkIdList*)`.
    pub fn get_cell_at_id_ijk_into_id_list(&self, ijk: [i32; 3], pt_ids: &mut IdList) {
        let values = self.cell_at_ijk(ijk);
        write_id_list(pt_ids, &values);
    }

    /// VTK: `vtkStructuredCellArray::GetCellAtId(int[3], vtkIdType&, vtkIdType*)`.
    pub fn get_cell_at_id_ijk_into_slice(&self, ijk: [i32; 3], cell_points: &mut [VtkIdType]) {
        let values = self.cell_at_ijk(ijk);
        assert!(
            cell_points.len() >= values.len(),
            "cell_points must have room for the structured cell"
        );
        cell_points[..values.len()].copy_from_slice(&values);
    }

    /// VTK: `vtkStructuredCellArray::GetCellSize`.
    pub fn get_cell_size(&self, _cell_id: VtkIdType) -> VtkIdType {
        self.storage().cell_size
    }

    /// VTK: `vtkStructuredCellArray::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        self.storage().cell_size as i32
    }

    /// VTK: `vtkStructuredCellArray::DeepCopy`.
    pub fn deep_copy(&mut self, ca: &Self) {
        self.connectivity = ca
            .connectivity
            .as_ref()
            .map(|storage| Arc::new((**storage).clone()));
        self.modified();
    }

    /// VTK: `vtkStructuredCellArray::ShallowCopy`.
    pub fn shallow_copy(&mut self, ca: &Self) {
        self.connectivity = ca.connectivity.as_ref().map(Arc::clone);
        self.modified();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.abstract_cell_array.get_class_name()
    }

    /// VTK: `vtkStructuredCellArray::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkStructuredCellArray" || name == "vtkAbstractCellArray"
    }

    /// VTK: `vtkStructuredCellArray::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkStructuredCellArray::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkStructuredCellArray" => 0,
            "vtkAbstractCellArray" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => -1,
        }
    }

    /// VTK: `vtkStructuredCellArray::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.abstract_cell_array.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.abstract_cell_array.get_m_time()
    }

    fn storage(&self) -> &StructuredCellArrayStorage {
        self.connectivity
            .as_deref()
            .expect("vtkStructuredCellArray connectivity is null; call set_data first")
    }

    fn cell_at_id(&self, cell_id: VtkIdType) -> Vec<VtkIdType> {
        let storage = self.storage();
        assert!(
            0 <= cell_id && cell_id < storage.number_of_cells,
            "cell id out of range"
        );
        map_tuple(storage, cell_id)
    }

    fn cell_at_ijk(&self, ijk: [i32; 3]) -> Vec<VtkIdType> {
        map_structured_tuple(self.storage(), ijk)
    }
}

impl Default for StructuredCellArray {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractCellArrayApi for StructuredCellArray {
    fn initialize(&mut self) {
        Self::initialize(self);
    }

    fn get_number_of_cells(&self) -> VtkIdType {
        Self::get_number_of_cells(self)
    }

    fn get_number_of_offsets(&self) -> VtkIdType {
        Self::get_number_of_offsets(self)
    }

    fn get_offset(&mut self, cell_id: VtkIdType) -> VtkIdType {
        Self::get_offset(self, cell_id)
    }

    fn get_number_of_connectivity_ids(&self) -> VtkIdType {
        Self::get_number_of_connectivity_ids(self)
    }

    fn is_storage_shareable(&self) -> bool {
        Self::is_storage_shareable(self)
    }

    fn is_homogeneous(&self) -> VtkIdType {
        Self::is_homogeneous(self)
    }

    fn get_cell_at_id_with_temp(
        &mut self,
        cell_id: VtkIdType,
        pt_ids: &mut IdList,
    ) -> Vec<VtkIdType> {
        Self::get_cell_at_id_with_temp(self, cell_id, pt_ids)
    }

    fn get_cell_at_id_into_id_list(&mut self, cell_id: VtkIdType, pts: &mut IdList) {
        Self::get_cell_at_id_into_id_list(self, cell_id, pts);
    }

    fn get_cell_at_id_into_slice(&mut self, cell_id: VtkIdType, cell_points: &mut [VtkIdType]) {
        Self::get_cell_at_id_into_slice(self, cell_id, cell_points);
    }

    fn get_cell_size(&self, cell_id: VtkIdType) -> VtkIdType {
        Self::get_cell_size(self, cell_id)
    }

    fn get_max_cell_size(&mut self) -> i32 {
        Self::get_max_cell_size(self)
    }

    fn deep_copy(&mut self, ca: AbstractCellArrayHandle) {
        if ca.is_null() {
            return;
        }
        let other = unsafe { &*(ca as *const Self) };
        Self::deep_copy(self, other);
    }

    fn shallow_copy(&mut self, ca: AbstractCellArrayHandle) {
        if ca.is_null() {
            return;
        }
        let other = unsafe { &*(ca as *const Self) };
        Self::shallow_copy(self, other);
    }
}

fn storage_from_extent(
    extent: [i32; 6],
    use_pixel_voxel_orientation: bool,
) -> StructuredCellArrayStorage {
    let dimensions = StructuredData::get_dimensions_from_extent(extent);
    let data_description = StructuredData::get_data_description(dimensions);
    StructuredCellArrayStorage {
        extent,
        dimensions,
        data_description,
        number_of_cells: StructuredData::get_number_of_cells(extent),
        cell_size: VtkIdType::from(cell_size(data_description)),
        use_pixel_voxel_orientation,
    }
}

fn empty_storage(use_pixel_voxel_orientation: bool) -> StructuredCellArrayStorage {
    StructuredCellArrayStorage {
        extent: [0, -1, 0, -1, 0, -1],
        dimensions: [0, 0, 0],
        data_description: VTK_STRUCTURED_EMPTY,
        number_of_cells: 0,
        cell_size: 0,
        use_pixel_voxel_orientation,
    }
}

fn map_tuple(storage: &StructuredCellArrayStorage, cell_id: VtkIdType) -> Vec<VtkIdType> {
    let ijk = cell_origin(cell_id, storage.data_description, storage.dimensions);
    map_structured_tuple(storage, ijk)
}

fn map_structured_tuple(storage: &StructuredCellArrayStorage, ijk: [i32; 3]) -> Vec<VtkIdType> {
    let shifts = shift_lut(
        storage.data_description,
        storage.use_pixel_voxel_orientation,
    );
    let mut values = Vec::with_capacity(storage.cell_size as usize);
    for comp in 0..storage.cell_size as usize {
        values.push(
            VtkIdType::from(ijk[0] + shifts[0][comp])
                + VtkIdType::from(ijk[1] + shifts[1][comp])
                    * VtkIdType::from(storage.dimensions[0])
                + VtkIdType::from(ijk[2] + shifts[2][comp])
                    * VtkIdType::from(storage.dimensions[0] * storage.dimensions[1]),
        );
    }
    values
}

fn cell_origin(cell_id: VtkIdType, data_description: i32, dimensions: [i32; 3]) -> [i32; 3] {
    match data_description {
        VTK_STRUCTURED_EMPTY | VTK_STRUCTURED_SINGLE_POINT => [0, 0, 0],
        VTK_STRUCTURED_X_LINE => [cell_id as i32, 0, 0],
        VTK_STRUCTURED_Y_LINE => [0, cell_id as i32, 0],
        VTK_STRUCTURED_Z_LINE => [0, 0, cell_id as i32],
        VTK_STRUCTURED_XY_PLANE => {
            let i = cell_id % VtkIdType::from(dimensions[0] - 1);
            let j = cell_id / VtkIdType::from(dimensions[0] - 1);
            [i as i32, j as i32, 0]
        }
        VTK_STRUCTURED_YZ_PLANE => {
            let j = cell_id % VtkIdType::from(dimensions[1] - 1);
            let k = cell_id / VtkIdType::from(dimensions[1] - 1);
            [0, j as i32, k as i32]
        }
        VTK_STRUCTURED_XZ_PLANE => {
            let i = cell_id % VtkIdType::from(dimensions[0] - 1);
            let k = cell_id / VtkIdType::from(dimensions[0] - 1);
            [i as i32, 0, k as i32]
        }
        VTK_STRUCTURED_XYZ_GRID => {
            let i = cell_id % VtkIdType::from(dimensions[0] - 1);
            let j =
                (cell_id / VtkIdType::from(dimensions[0] - 1)) % VtkIdType::from(dimensions[1] - 1);
            let k =
                cell_id / (VtkIdType::from(dimensions[0] - 1) * VtkIdType::from(dimensions[1] - 1));
            [i as i32, j as i32, k as i32]
        }
        _ => [0, 0, 0],
    }
}

fn cell_size(data_description: i32) -> i32 {
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

fn write_id_list(ids: &mut IdList, values: &[VtkIdType]) {
    ids.reset();
    ids.set_number_of_ids(values.len() as VtkIdType);
    for (idx, value) in values.iter().enumerate() {
        ids.set_id(idx as VtkIdType, *value);
    }
}

fn id_list_values(ids: &IdList) -> Vec<VtkIdType> {
    (0..ids.get_number_of_ids())
        .map(|idx| ids.get_id(idx))
        .collect()
}
