use crate::common::core::{IdList, IdTypeArray, VtkIdType};

use super::{CellArrayIterator, CellBaseApi};

use std::fmt::Write;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct CellArrayStorage {
    offsets: IdTypeArray,
    connectivity: IdTypeArray,
    storage_type: CellArrayStorageType,
}

/// Storage for cell topology using offsets plus connectivity arrays.
///
/// VTK origin: `VTK/Common/DataModel/vtkCellArray.cxx`.
#[derive(Debug)]
pub struct CellArray {
    storage: Arc<CellArrayStorage>,
    traversal_cell_id: VtkIdType,
    legacy_data: IdTypeArray,
}

/// VTK origin: `vtkCellArray::StorageTypes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CellArrayStorageType {
    Int64 = 0,
    Int32 = 1,
    FixedSizeInt64 = 2,
    FixedSizeInt32 = 3,
    Generic = 4,
}

impl Clone for CellArray {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            traversal_cell_id: self.traversal_cell_id,
            legacy_data: self.legacy_data.clone(),
        }
    }
}

impl PartialEq for CellArray {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage) || self.storage == other.storage
    }
}

impl CellArray {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(empty_storage()),
            traversal_cell_id: 0,
            legacy_data: IdTypeArray::new(),
        }
    }

    fn storage(&self) -> &CellArrayStorage {
        &self.storage
    }

    fn storage_mut(&mut self) -> &mut CellArrayStorage {
        Arc::make_mut(&mut self.storage)
    }

    fn push_cell(&mut self, point_ids: &[i64]) {
        let storage = self.storage_mut();
        append_values(&mut storage.connectivity, point_ids);
        storage
            .offsets
            .insert_next_typed_tuple(&[storage.connectivity.get_number_of_values() as i64]);
    }

    /// VTK: `vtkCellArray::InsertNextCell`.
    pub fn insert_next_cell(&mut self, point_ids: &[VtkIdType]) -> VtkIdType {
        let id = self.get_number_of_cells();
        self.push_cell(point_ids);
        id
    }

    /// VTK: `vtkCellArray::InsertNextCell(int npts)`.
    pub fn insert_next_cell_empty(&mut self, npts: i32) -> VtkIdType {
        assert!(npts >= 0, "number of points must be non-negative");
        let cell_id = self.get_number_of_cells();
        let next_offset = self
            .storage
            .connectivity
            .get_number_of_values()
            .saturating_add(VtkIdType::from(npts));
        self.storage_mut()
            .offsets
            .insert_next_typed_tuple(&[next_offset]);
        cell_id
    }

    /// VTK: `vtkCellArray::InsertNextCell(vtkIdList*)`.
    pub fn insert_next_cell_from_id_list(&mut self, point_ids: &IdList) -> VtkIdType {
        let ids: Vec<_> = (0..point_ids.get_number_of_ids())
            .map(|i| point_ids.get_id(i))
            .collect();
        self.insert_next_cell(&ids)
    }

    /// VTK: `vtkCellArray::InsertNextCell(vtkCell*)`.
    pub fn insert_next_cell_from_cell<C: CellBaseApi + ?Sized>(&mut self, cell: &C) -> VtkIdType {
        self.insert_next_cell_from_id_list(cell.cell().get_point_ids())
    }

    /// VTK: `vtkCellArray::InsertCellPoint`.
    pub fn insert_cell_point(&mut self, point_id: VtkIdType) {
        self.storage_mut()
            .connectivity
            .insert_next_typed_tuple(&[point_id]);
    }

    /// VTK: `vtkCellArray::UpdateCellCount`.
    pub fn update_cell_count(&mut self, npts: i32) {
        assert!(npts >= 0, "number of points must be non-negative");
        let storage = self.storage_mut();
        let offsets = storage.offsets.as_mut_slice();
        let len = offsets.len();
        assert!(len >= 2, "no open cell to update");
        offsets[len - 1] = offsets[len - 2] + VtkIdType::from(npts);
    }

    /// VTK: `vtkCellArray::Append`.
    pub fn append(&mut self, other: &Self) {
        self.append_with_point_offset(other, 0);
    }

    /// VTK: `vtkCellArray::Append(vtkCellArray*, vtkIdType)`.
    pub fn append_with_point_offset(&mut self, other: &Self, point_offset: VtkIdType) {
        let other_storage = other.storage();
        if other_storage.offsets.get_number_of_values() <= 1 {
            return;
        }
        let shifted_connectivity: Vec<_> = other_storage
            .connectivity
            .as_slice()
            .iter()
            .map(|point_id| point_id + point_offset)
            .collect();
        let storage = self.storage_mut();
        let base = storage.connectivity.get_number_of_values() as i64;
        append_values(&mut storage.connectivity, &shifted_connectivity);

        storage.offsets.reserve_values(
            other_storage
                .offsets
                .get_number_of_values()
                .saturating_sub(1),
        );
        for offset in other_storage.offsets.as_slice().iter().skip(1) {
            storage.offsets.insert_next_typed_tuple(&[offset + base]);
        }
    }

    /// VTK: `vtkCellArray::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new((*other.storage).clone());
    }

    /// VTK: `vtkCellArray::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        if !Arc::ptr_eq(&self.storage, &other.storage) {
            self.storage = Arc::clone(&other.storage);
        }
    }

    /// VTK: `vtkCellArray::AppendLegacyFormat`.
    pub fn append_legacy_format(&mut self, legacy: &[VtkIdType], pt_offset: VtkIdType) {
        let Some(cells) = parse_legacy_cells(legacy, pt_offset) else {
            return;
        };
        for cell in cells {
            self.push_cell(&cell);
        }
    }

    /// VTK: `vtkCellArray::ImportLegacyFormat`.
    pub fn import_legacy_format(&mut self, legacy: &[VtkIdType]) {
        let mut cells = Self::new();
        cells.storage_mut().storage_type = self.storage.storage_type;
        cells.append_legacy_format(legacy, 0);
        self.storage = cells.storage;
    }

    /// VTK: `vtkCellArray::GetData`.
    pub fn get_data(&mut self) -> &IdTypeArray {
        let mut legacy_data = IdTypeArray::new();
        self.export_legacy_format(&mut legacy_data);
        self.legacy_data = legacy_data;
        &self.legacy_data
    }

    /// VTK: `vtkCellArray::SetCells`.
    pub fn set_cells(&mut self, number_of_cells: VtkIdType, cells: &IdTypeArray) {
        self.allocate_exact(
            number_of_cells,
            cells.get_number_of_values().saturating_sub(number_of_cells),
        );
        self.import_legacy_format(cells.as_slice());
    }

    /// VTK: `vtkCellArray::SetData(vtkCellArray::AOSArray64*, vtkCellArray::AOSArray64*)`.
    pub fn set_data(&mut self, offsets: &IdTypeArray, connectivity: &IdTypeArray) -> bool {
        if offsets.get_number_of_components() != 1 || connectivity.get_number_of_components() != 1 {
            return false;
        }
        self.storage = Arc::new(CellArrayStorage {
            offsets: offsets.clone(),
            connectivity: connectivity.clone(),
            storage_type: CellArrayStorageType::Int64,
        });
        true
    }

    /// VTK: `vtkCellArray::SetData(vtkIdType cellSize, vtkDataArray* connectivity)`.
    pub fn set_data_with_fixed_cell_size(
        &mut self,
        cell_size: VtkIdType,
        connectivity: &IdTypeArray,
    ) -> bool {
        if cell_size <= 0 || connectivity.get_number_of_values() % cell_size != 0 {
            return false;
        }
        let number_of_cells = connectivity.get_number_of_values() / cell_size;
        let offsets: Vec<_> = (0..=number_of_cells)
            .map(|cell_id| cell_id * cell_size)
            .collect();
        let offsets = IdTypeArray::from_vec("Offsets", offsets, 1);
        self.storage = Arc::new(CellArrayStorage {
            offsets,
            connectivity: connectivity.clone(),
            storage_type: CellArrayStorageType::FixedSizeInt64,
        });
        true
    }

    /// VTK: `vtkCellArray::ExportLegacyFormat`.
    pub fn export_legacy_format(&self, data: &mut IdTypeArray) {
        data.initialize();
        data.set_number_of_components(1);
        data.reserve_values(self.get_number_of_connectivity_entries());
        for cell in self.iter() {
            data.insert_next_typed_tuple(&[cell.len() as VtkIdType]);
            for &point_id in cell {
                data.insert_next_typed_tuple(&[point_id]);
            }
        }
    }

    fn number_of_cells(&self) -> usize {
        self.storage
            .offsets
            .get_number_of_values()
            .saturating_sub(1) as usize
    }

    /// VTK: `vtkCellArray::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        self.number_of_cells() as VtkIdType
    }

    /// VTK: `vtkCellArray::SetNumberOfCells`.
    pub fn set_number_of_cells(&mut self, _number_of_cells: VtkIdType) {
        // VTK keeps this deprecated API as a no-op. Cell count is derived from
        // the offsets array and should be changed by inserting or setting data.
    }

    /// VTK: `vtkCellArray::GetNumberOfOffsets`.
    pub fn get_number_of_offsets(&self) -> VtkIdType {
        self.storage.offsets.get_number_of_values()
    }

    /// VTK: `vtkCellArray::GetOffset`.
    pub fn get_offset(&self, cell_id: VtkIdType) -> VtkIdType {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        *self
            .storage
            .offsets
            .as_slice()
            .get(cell_id)
            .expect("cell id out of range")
    }

    /// VTK: `vtkCellArray::SetOffset`.
    pub fn set_offset(&mut self, cell_id: VtkIdType, offset: VtkIdType) {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        let offsets = self.storage_mut().offsets.as_mut_slice();
        *offsets.get_mut(cell_id).expect("cell id out of range") = offset;
    }

    fn cell_unchecked(&self, idx: usize) -> &[i64] {
        let offsets = self.storage.offsets.as_slice();
        let start = offsets[idx] as usize;
        let end = offsets[idx + 1] as usize;
        &self.storage.connectivity.as_slice()[start..end]
    }

    fn location_to_cell_id(&self, location: VtkIdType) -> Option<VtkIdType> {
        let offsets = self.storage.offsets.as_slice();
        if offsets.len() <= 1 {
            return None;
        }
        let mut begin = 0usize;
        let mut len = offsets.len() - 1;
        while len > 0 {
            let step = len / 2;
            let mid = begin + step;
            let current_location = offsets[mid] + mid as VtkIdType;
            if current_location < location {
                begin = mid + 1;
                len -= step + 1;
            } else {
                len = step;
            }
        }
        if begin < offsets.len() - 1 && offsets[begin] + begin as VtkIdType == location {
            Some(begin as VtkIdType)
        } else {
            None
        }
    }

    fn cell_id_to_location(&self, cell_id: VtkIdType) -> VtkIdType {
        self.get_offset(cell_id) + cell_id
    }

    /// VTK: `vtkCellArray::GetCell`.
    pub fn get_cell(&self, location: VtkIdType) -> Option<&[VtkIdType]> {
        let cell_id = self.location_to_cell_id(location)?;
        Some(self.get_cell_at_id(cell_id))
    }

    /// VTK: `vtkCellArray::GetCell`.
    pub fn get_cell_into_id_list(&self, location: VtkIdType, point_ids: &mut IdList) -> bool {
        let Some(cell) = self.get_cell(location) else {
            point_ids.reset();
            return false;
        };
        point_ids.set_number_of_ids(cell.len() as VtkIdType);
        for (i, point_id) in cell.iter().enumerate() {
            point_ids.set_id(i as VtkIdType, *point_id);
        }
        true
    }

    /// VTK: `vtkCellArray::GetCellAtId`.
    pub fn get_cell_at_id(&self, cell_id: VtkIdType) -> &[VtkIdType] {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        assert!(cell_id < self.number_of_cells(), "cell id out of range");
        self.cell_unchecked(cell_id)
    }

    /// VTK: `vtkCellArray::GetCellAtId`.
    pub fn get_cell_at_id_into_id_list(&self, cell_id: VtkIdType, point_ids: &mut IdList) {
        let cell = self.get_cell_at_id(cell_id);
        point_ids.set_number_of_ids(cell.len() as VtkIdType);
        for (i, point_id) in cell.iter().enumerate() {
            point_ids.set_id(i as VtkIdType, *point_id);
        }
    }

    /// VTK: `vtkCellArray::GetCellPointAtId`.
    pub fn get_cell_point_at_id(&self, cell_id: VtkIdType, point_id: VtkIdType) -> VtkIdType {
        let point_id = vtk_id_to_index(point_id).expect("cell point id must be non-negative");
        self.get_cell_at_id(cell_id)
            .get(point_id)
            .copied()
            .expect("cell point id out of range")
    }

    /// VTK: `vtkCellArray::ReverseCellAtId`.
    pub fn reverse_cell_at_id(&mut self, cell_id: VtkIdType) {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        assert!(cell_id < self.number_of_cells(), "cell id out of range");
        let storage = self.storage_mut();
        let offsets = storage.offsets.as_slice();
        let start = offsets[cell_id] as usize;
        let end = offsets[cell_id + 1] as usize;
        storage.connectivity.as_mut_slice()[start..end].reverse();
    }

    /// VTK: `vtkCellArray::ReverseCell`.
    pub fn reverse_cell(&mut self, location: VtkIdType) {
        if let Some(cell_id) = self.location_to_cell_id(location) {
            self.reverse_cell_at_id(cell_id);
        }
    }

    /// VTK: `vtkCellArray::ReplaceCellAtId`.
    pub fn replace_cell_at_id(&mut self, cell_id: VtkIdType, point_ids: &[VtkIdType]) {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        assert!(cell_id < self.number_of_cells(), "cell id out of range");

        let offsets = self.storage.offsets.as_slice();
        let start = offsets[cell_id] as usize;
        let end = offsets[cell_id + 1] as usize;
        assert_eq!(point_ids.len(), end - start);

        let storage = self.storage_mut();
        storage.connectivity.as_mut_slice()[start..end].copy_from_slice(point_ids);
    }

    /// VTK: `vtkCellArray::ReplaceCellAtId(vtkIdList*)`.
    pub fn replace_cell_at_id_from_id_list(&mut self, cell_id: VtkIdType, point_ids: &IdList) {
        let ids: Vec<_> = (0..point_ids.get_number_of_ids())
            .map(|i| point_ids.get_id(i))
            .collect();
        self.replace_cell_at_id(cell_id, &ids);
    }

    /// VTK: `vtkCellArray::ReplaceCell`.
    pub fn replace_cell(&mut self, location: VtkIdType, point_ids: &[VtkIdType]) {
        if let Some(cell_id) = self.location_to_cell_id(location) {
            self.replace_cell_at_id(cell_id, point_ids);
        }
    }

    /// VTK: `vtkCellArray::ReplaceCellPointAtId`.
    pub fn replace_cell_point_at_id(
        &mut self,
        cell_id: VtkIdType,
        cell_point_index: VtkIdType,
        new_point_id: VtkIdType,
    ) {
        let cell_id = vtk_id_to_index(cell_id).expect("cell id must be non-negative");
        assert!(cell_id < self.number_of_cells(), "cell id out of range");
        let cell_point_index =
            vtk_id_to_index(cell_point_index).expect("cell point index must be non-negative");
        assert!(
            cell_point_index < self.cell_size(cell_id),
            "cell point index out of range"
        );

        let idx = self.storage.offsets.as_slice()[cell_id] as usize + cell_point_index;
        self.storage_mut().connectivity.as_mut_slice()[idx] = new_point_id;
    }

    fn offsets(&self) -> &[i64] {
        self.storage.offsets.as_slice()
    }

    /// VTK: `vtkCellArray::GetOffsetsArray`.
    pub fn get_offsets_array(&self) -> &[VtkIdType] {
        self.offsets()
    }

    fn connectivity(&self) -> &[i64] {
        self.storage.connectivity.as_slice()
    }

    /// VTK: `vtkCellArray::GetConnectivityArray`.
    pub fn get_connectivity_array(&self) -> &[VtkIdType] {
        self.connectivity()
    }

    fn connectivity_len(&self) -> VtkIdType {
        self.storage.connectivity.get_number_of_values()
    }

    /// VTK: `vtkCellArray::GetNumberOfConnectivityIds`.
    pub fn get_number_of_connectivity_ids(&self) -> VtkIdType {
        self.connectivity_len()
    }

    /// VTK: `vtkCellArray::GetNumberOfConnectivityEntries`.
    pub fn get_number_of_connectivity_entries(&self) -> VtkIdType {
        self.storage.connectivity.get_number_of_values() + self.get_number_of_cells()
    }

    /// VTK: `vtkCellArray::GetSize`.
    pub fn get_size(&self) -> VtkIdType {
        (self.storage.offsets.capacity() + self.storage.connectivity.capacity()) as VtkIdType
    }

    /// VTK: `vtkCellArray::GetInsertLocation`.
    pub fn get_insert_location(&self, npts: i32) -> VtkIdType {
        self.storage.connectivity.get_number_of_values() + self.get_number_of_cells()
            - VtkIdType::from(npts)
            - 1
    }

    /// VTK: `vtkCellArray::GetTraversalCellId`.
    pub fn get_traversal_cell_id(&self) -> VtkIdType {
        self.traversal_cell_id
    }

    /// VTK: `vtkCellArray::SetTraversalCellId`.
    pub fn set_traversal_cell_id(&mut self, cell_id: VtkIdType) {
        self.traversal_cell_id = cell_id;
    }

    /// VTK: `vtkCellArray::GetTraversalLocation`.
    pub fn get_traversal_location(&self) -> VtkIdType {
        self.cell_id_to_location(self.traversal_cell_id)
    }

    /// VTK: `vtkCellArray::GetTraversalLocation(vtkIdType npts)`.
    pub fn get_traversal_location_with_npts(&self, npts: VtkIdType) -> VtkIdType {
        self.get_traversal_location() - npts - 1
    }

    /// VTK: `vtkCellArray::SetTraversalLocation`.
    pub fn set_traversal_location(&mut self, location: VtkIdType) {
        if let Some(cell_id) = self.location_to_cell_id(location) {
            self.set_traversal_cell_id(cell_id);
        }
    }

    /// VTK: `vtkCellArray::AllocateExact`.
    pub fn allocate_exact(
        &mut self,
        number_of_cells: VtkIdType,
        connectivity_size: VtkIdType,
    ) -> bool {
        let number_of_cells =
            vtk_id_to_index(number_of_cells).expect("number of cells must be non-negative");
        let connectivity_size =
            vtk_id_to_index(connectivity_size).expect("connectivity size must be non-negative");
        let storage = self.storage_mut();
        storage.offsets = empty_id_type_array("Offsets", number_of_cells + 1);
        storage.offsets.insert_next_typed_tuple(&[0]);
        storage.connectivity = empty_id_type_array("Connectivity", connectivity_size);
        true
    }

    /// VTK: `vtkCellArray::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, _ext: VtkIdType) -> bool {
        self.allocate_exact(size, size)
    }

    /// VTK: `vtkCellArray::AllocateEstimate`.
    pub fn allocate_estimate(
        &mut self,
        number_of_cells: VtkIdType,
        max_cell_size: VtkIdType,
    ) -> bool {
        self.allocate_exact(
            number_of_cells,
            number_of_cells.saturating_mul(max_cell_size),
        )
    }

    /// VTK: `vtkCellArray::AllocateCopy`.
    pub fn allocate_copy(&mut self, other: &Self) -> bool {
        self.allocate_exact(
            other.get_number_of_cells(),
            other.get_number_of_connectivity_ids(),
        )
    }

    /// VTK: `vtkCellArray::ResizeExact`.
    pub fn resize_exact(
        &mut self,
        number_of_cells: VtkIdType,
        connectivity_size: VtkIdType,
    ) -> bool {
        let number_of_cells =
            vtk_id_to_index(number_of_cells).expect("number of cells must be non-negative");
        let connectivity_size =
            vtk_id_to_index(connectivity_size).expect("connectivity size must be non-negative");
        let storage = self.storage_mut();
        storage
            .connectivity
            .set_number_of_values(connectivity_size as VtkIdType);
        let conn_len = storage.connectivity.get_number_of_values();
        storage
            .offsets
            .set_number_of_values((number_of_cells + 1) as VtkIdType);
        for offset in storage.offsets.as_mut_slice() {
            *offset = (*offset).min(conn_len);
        }
        *storage
            .offsets
            .as_mut_slice()
            .last_mut()
            .expect("offsets always contains a sentinel") = conn_len;
        true
    }

    fn clear(&mut self) {
        let storage = self.storage_mut();
        storage.offsets.reset();
        storage.offsets.insert_next_typed_tuple(&[0]);
        storage.connectivity.reset();
    }

    /// VTK: `vtkCellArray::Reset`.
    pub fn reset(&mut self) {
        self.clear();
    }

    /// VTK: `vtkCellArray::Initialize`.
    pub fn initialize(&mut self) {
        self.storage = Arc::new(empty_storage_with_type(self.storage.storage_type));
    }

    /// VTK: `vtkCellArray::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.offsets.squeeze();
        storage.connectivity.squeeze();
    }

    /// VTK: `vtkCellArray::IsValid`.
    pub fn is_valid(&self) -> bool {
        let offsets = self.storage.offsets.as_slice();
        let connectivity = self.storage.connectivity.as_slice();
        if offsets.first() != Some(&0) {
            return false;
        }
        let conn_len = connectivity.len() as i64;
        offsets.windows(2).all(|pair| pair[0] <= pair[1])
            && offsets
                .iter()
                .all(|&offset| (0..=conn_len).contains(&offset))
            && offsets.last() == Some(&conn_len)
    }

    fn cell_size(&self, idx: usize) -> usize {
        let offsets = self.storage.offsets.as_slice();
        let start = offsets[idx] as usize;
        let end = offsets[idx + 1] as usize;
        end - start
    }

    /// VTK: `vtkCellArray::GetCellSize`.
    pub fn get_cell_size(&self, idx: VtkIdType) -> VtkIdType {
        let idx = vtk_id_to_index(idx).expect("cell id must be non-negative");
        assert!(idx < self.number_of_cells(), "cell id out of range");
        self.cell_size(idx) as VtkIdType
    }

    /// VTK: `vtkCellArray::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.traversal_cell_id = 0;
    }

    /// VTK: `vtkCellArray::GetNextCell`.
    pub fn get_next_cell(&mut self) -> Option<&[VtkIdType]> {
        if self.traversal_cell_id < self.get_number_of_cells() {
            let cell_id = self.traversal_cell_id as usize;
            self.traversal_cell_id += 1;
            Some(self.cell_unchecked(cell_id))
        } else {
            None
        }
    }

    /// VTK: `vtkCellArray::GetNextCell(vtkIdList*)`.
    pub fn get_next_cell_into_id_list(&mut self, point_ids: &mut IdList) -> bool {
        if self.traversal_cell_id < self.get_number_of_cells() {
            let cell_id = self.traversal_cell_id;
            self.traversal_cell_id += 1;
            self.get_cell_at_id_into_id_list(cell_id, point_ids);
            true
        } else {
            point_ids.reset();
            false
        }
    }

    /// VTK: `vtkCellArray::EstimateSize`.
    pub fn estimate_size(number_of_cells: VtkIdType, max_cell_size: i32) -> VtkIdType {
        number_of_cells * VtkIdType::from(max_cell_size + 1)
    }

    fn cell_sizes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.number_of_cells()).map(move |i| self.cell_size(i))
    }

    /// VTK: `vtkCellArray::GetMaxCellSize`.
    pub fn get_max_cell_size(&self) -> i32 {
        self.cell_sizes().max().unwrap_or(0) as i32
    }

    /// VTK: `vtkCellArray::IsHomogeneous`.
    pub fn is_homogeneous(&self) -> VtkIdType {
        let mut sizes = self.cell_sizes();
        let Some(first) = sizes.next() else {
            return 0;
        };
        if sizes.all(|size| size == first) {
            first as VtkIdType
        } else {
            -1
        }
    }

    /// VTK: `vtkCellArray::CanConvertTo64BitStorage`.
    pub fn can_convert_to_64_bit_storage(&self) -> bool {
        true
    }

    /// VTK: `vtkCellArray::CanConvertToDefaultStorage`.
    pub fn can_convert_to_default_storage(&self) -> bool {
        true
    }

    /// VTK: `vtkCellArray::CanConvertTo32BitStorage`.
    pub fn can_convert_to_32_bit_storage(&self) -> bool {
        self.storage
            .offsets
            .as_slice()
            .iter()
            .chain(self.storage.connectivity.as_slice().iter())
            .all(|&value| i32::try_from(value).is_ok())
    }

    /// VTK: `vtkCellArray::CanConvertToFixedSize64BitStorage`.
    pub fn can_convert_to_fixed_size_64_bit_storage(&self) -> bool {
        self.is_homogeneous() >= 0
    }

    /// VTK: `vtkCellArray::CanConvertToFixedSizeDefaultStorage`.
    pub fn can_convert_to_fixed_size_default_storage(&self) -> bool {
        self.can_convert_to_fixed_size_64_bit_storage()
    }

    /// VTK: `vtkCellArray::CanConvertToFixedSize32BitStorage`.
    pub fn can_convert_to_fixed_size_32_bit_storage(&self) -> bool {
        self.can_convert_to_32_bit_storage() && self.is_homogeneous() >= 0
    }

    /// VTK: `vtkCellArray::CanConvertToStorageType`.
    pub fn can_convert_to_storage_type(&self, storage_type: CellArrayStorageType) -> bool {
        match storage_type {
            CellArrayStorageType::Int32 => self.can_convert_to_32_bit_storage(),
            CellArrayStorageType::Int64 => self.can_convert_to_64_bit_storage(),
            CellArrayStorageType::FixedSizeInt32 => self.can_convert_to_fixed_size_32_bit_storage(),
            CellArrayStorageType::FixedSizeInt64 => self.can_convert_to_fixed_size_64_bit_storage(),
            CellArrayStorageType::Generic => true,
        }
    }

    /// VTK: `vtkCellArray::ConvertTo64BitStorage`.
    pub fn convert_to_64_bit_storage(&mut self) -> bool {
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::Int64;
        true
    }

    /// VTK: `vtkCellArray::ConvertToDefaultStorage`.
    pub fn convert_to_default_storage(&mut self) -> bool {
        self.convert_to_64_bit_storage()
    }

    /// VTK: `vtkCellArray::ConvertTo32BitStorage`.
    pub fn convert_to_32_bit_storage(&mut self) -> bool {
        if !self.can_convert_to_32_bit_storage() {
            return false;
        }
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::Int32;
        true
    }

    /// VTK: `vtkCellArray::ConvertToFixedSize64BitStorage`.
    pub fn convert_to_fixed_size_64_bit_storage(&mut self) -> bool {
        let cell_size = self.is_homogeneous();
        if cell_size < 0 {
            return false;
        }
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::FixedSizeInt64;
        true
    }

    /// VTK: `vtkCellArray::ConvertToFixedSizeDefaultStorage`.
    pub fn convert_to_fixed_size_default_storage(&mut self) -> bool {
        self.convert_to_fixed_size_64_bit_storage()
    }

    /// VTK: `vtkCellArray::ConvertToFixedSize32BitStorage`.
    pub fn convert_to_fixed_size_32_bit_storage(&mut self) -> bool {
        if !self.can_convert_to_fixed_size_32_bit_storage() {
            return false;
        }
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::FixedSizeInt32;
        true
    }

    /// VTK: `vtkCellArray::ConvertToSmallestStorage`.
    pub fn convert_to_smallest_storage(&mut self) -> bool {
        if self.can_convert_to_fixed_size_32_bit_storage() {
            self.convert_to_fixed_size_32_bit_storage()
        } else if self.can_convert_to_32_bit_storage() {
            self.convert_to_32_bit_storage()
        } else if self.can_convert_to_fixed_size_64_bit_storage() {
            self.convert_to_fixed_size_64_bit_storage()
        } else {
            self.convert_to_64_bit_storage()
        }
    }

    /// VTK: `vtkCellArray::ConvertToStorageType`.
    pub fn convert_to_storage_type(&mut self, storage_type: CellArrayStorageType) -> bool {
        match storage_type {
            CellArrayStorageType::Int32 => self.convert_to_32_bit_storage(),
            CellArrayStorageType::Int64 => self.convert_to_64_bit_storage(),
            CellArrayStorageType::FixedSizeInt32 => self.convert_to_fixed_size_32_bit_storage(),
            CellArrayStorageType::FixedSizeInt64 => self.convert_to_fixed_size_64_bit_storage(),
            CellArrayStorageType::Generic => {
                let storage = self.storage_mut();
                storage.storage_type = CellArrayStorageType::Generic;
                true
            }
        }
    }

    /// VTK: `vtkCellArray::GetStorageType`.
    pub fn get_storage_type(&self) -> CellArrayStorageType {
        self.storage.storage_type
    }

    /// VTK: `vtkCellArray::IsStorage64Bit`.
    pub fn is_storage_64_bit(&self) -> bool {
        self.storage.storage_type == CellArrayStorageType::Int64
    }

    /// VTK: `vtkCellArray::IsStorage32Bit`.
    pub fn is_storage_32_bit(&self) -> bool {
        self.storage.storage_type == CellArrayStorageType::Int32
    }

    /// VTK: `vtkCellArray::IsStorageFixedSize64Bit`.
    pub fn is_storage_fixed_size_64_bit(&self) -> bool {
        self.storage.storage_type == CellArrayStorageType::FixedSizeInt64
    }

    /// VTK: `vtkCellArray::IsStorageFixedSize32Bit`.
    pub fn is_storage_fixed_size_32_bit(&self) -> bool {
        self.storage.storage_type == CellArrayStorageType::FixedSizeInt32
    }

    /// VTK: `vtkCellArray::IsStorageFixedSize`.
    pub fn is_storage_fixed_size(&self) -> bool {
        self.is_storage_fixed_size_32_bit() || self.is_storage_fixed_size_64_bit()
    }

    /// VTK: `vtkCellArray::IsStorageGeneric`.
    pub fn is_storage_generic(&self) -> bool {
        self.storage.storage_type == CellArrayStorageType::Generic
    }

    /// VTK: `vtkCellArray::IsStorageShareable`.
    pub fn is_storage_shareable(&self) -> bool {
        self.is_storage_64_bit() || self.is_storage_fixed_size_64_bit()
    }

    /// VTK: `vtkCellArray::Use64BitStorage`.
    pub fn use_64_bit_storage(&mut self) {
        self.initialize();
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::Int64;
    }

    /// VTK: `vtkCellArray::UseDefaultStorage`.
    pub fn use_default_storage(&mut self) {
        self.use_64_bit_storage();
    }

    /// VTK: `vtkCellArray::Use32BitStorage`.
    pub fn use_32_bit_storage(&mut self) {
        self.initialize();
        let storage = self.storage_mut();
        storage.storage_type = CellArrayStorageType::Int32;
    }

    /// VTK: `vtkCellArray::UseFixedSize64BitStorage`.
    pub fn use_fixed_size_64_bit_storage(&mut self, _cell_size: VtkIdType) {
        self.initialize();
        self.storage_mut().storage_type = CellArrayStorageType::FixedSizeInt64;
    }

    /// VTK: `vtkCellArray::UseFixedSizeDefaultStorage`.
    pub fn use_fixed_size_default_storage(&mut self, cell_size: VtkIdType) {
        self.use_fixed_size_64_bit_storage(cell_size);
    }

    /// VTK: `vtkCellArray::UseFixedSize32BitStorage`.
    pub fn use_fixed_size_32_bit_storage(&mut self, _cell_size: VtkIdType) {
        self.initialize();
        self.storage_mut().storage_type = CellArrayStorageType::FixedSizeInt32;
    }

    /// VTK: `vtkCellArray::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.offsets.get_actual_memory_size()
            + self.storage.connectivity.get_actual_memory_size()
    }

    /// VTK: `vtkCellArray::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkCellArray {{ StorageType: {}, Offsets: {{ number_of_values: {}, actual_memory_size_kib: {} }}, Connectivity: {{ number_of_values: {}, actual_memory_size_kib: {} }} }}",
            storage_type_name(self.storage.storage_type),
            self.storage.offsets.get_number_of_values(),
            self.storage.offsets.get_actual_memory_size(),
            self.storage.connectivity.get_number_of_values(),
            self.storage.connectivity.get_actual_memory_size()
        )
    }

    /// VTK: `vtkCellArray::PrintDebug`.
    pub fn print_debug(&self) -> String {
        let mut out = self.print_self();
        out.push('\n');
        for (cell_id, cell) in self.iter().enumerate() {
            let _ = write!(out, "cell {cell_id}: ");
            for point_id in cell {
                let _ = write!(out, "{point_id} ");
            }
            out.push('\n');
        }
        out
    }

    /// VTK: `vtkCellArray::NewIterator`.
    pub fn new_iterator(&mut self) -> CellArrayIterator {
        let mut iterator = CellArrayIterator::new();
        iterator.set_cell_array(self as *mut Self);
        iterator.go_to_first_cell();
        iterator
    }

    pub(crate) fn slice_iterator(&self) -> CellIter<'_> {
        CellIter {
            cells: self,
            idx: 0,
        }
    }

    pub(crate) fn iter(&self) -> CellIter<'_> {
        self.slice_iterator()
    }
}

fn empty_storage() -> CellArrayStorage {
    empty_storage_with_type(CellArrayStorageType::Int64)
}

fn empty_storage_with_type(storage_type: CellArrayStorageType) -> CellArrayStorage {
    CellArrayStorage {
        offsets: id_type_array("Offsets", vec![0]),
        connectivity: id_type_array("Connectivity", Vec::new()),
        storage_type,
    }
}

fn storage_type_name(storage_type: CellArrayStorageType) -> &'static str {
    match storage_type {
        CellArrayStorageType::Int32 => "Int32",
        CellArrayStorageType::Int64 => "Int64",
        CellArrayStorageType::FixedSizeInt32 => "FixedSizeInt32",
        CellArrayStorageType::FixedSizeInt64 => "FixedSizeInt64",
        CellArrayStorageType::Generic => "Generic",
    }
}

fn id_type_array(name: &str, values: Vec<i64>) -> IdTypeArray {
    IdTypeArray::from_vec(name, values, 1)
}

fn empty_id_type_array(name: &str, capacity: usize) -> IdTypeArray {
    let mut array = IdTypeArray::with_name_and_number_of_components(name, 1);
    array.reserve_values(capacity as i64);
    array
}

fn append_values(array: &mut IdTypeArray, values: &[i64]) {
    array.reserve_values(array.get_number_of_values() + values.len() as i64);
    for value in values {
        array.insert_next_typed_tuple(&[*value]);
    }
}

fn vtk_id_to_index(id: VtkIdType) -> Option<usize> {
    usize::try_from(id).ok()
}

fn parse_legacy_cells(legacy: &[i64], pt_offset: i64) -> Option<Vec<Vec<i64>>> {
    let mut cells = Vec::new();
    let mut pos = 0;
    while pos < legacy.len() {
        let npts = legacy[pos];
        if npts < 0 {
            return None;
        }
        let npts = npts as usize;
        let start = pos + 1;
        let end = start.checked_add(npts)?;
        if end > legacy.len() {
            return None;
        }
        let mut cell = Vec::with_capacity(npts);
        for point_id in &legacy[start..end] {
            cell.push(point_id.checked_add(pt_offset)?);
        }
        cells.push(cell);
        pos = end;
    }
    Some(cells)
}

pub struct CellIter<'a> {
    cells: &'a CellArray,
    idx: usize,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = &'a [VtkIdType];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.cells.number_of_cells() {
            return None;
        }
        let cell = self.cells.cell_unchecked(self.idx);
        self.idx += 1;
        Some(cell)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.cells.number_of_cells() - self.idx;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CellIter<'_> {}
