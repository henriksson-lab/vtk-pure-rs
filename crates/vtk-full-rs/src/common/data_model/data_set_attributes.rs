use crate::common::{
    core::{AnyArray, VtkDataType, VtkIdType},
    data_model::{
        data_set_attributes_field_list::DataSetAttributesFieldList, FieldData, FieldDataArray,
    },
};
use std::sync::Arc;

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}

/// Active attribute roles from `vtkDataSetAttributes`.
///
/// VTK origin: `VTK/Common/DataModel/vtkDataSetAttributes.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSetAttribute {
    Scalars,
    Vectors,
    Normals,
    TCoords,
    Tensors,
    GlobalIds,
    PedigreeIds,
    EdgeFlag,
    Tangents,
    RationalWeights,
    HigherOrderDegrees,
    ProcessIds,
}

/// Copy operation selector from `vtkDataSetAttributes::AttributeCopyOperations`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSetAttributeCopyOperation {
    CopyTuple,
    Interpolate,
    PassData,
    AllCopy,
}

impl DataSetAttributeCopyOperation {
    const SINGLE: [Self; 3] = [Self::CopyTuple, Self::Interpolate, Self::PassData];

    const fn from_i32(value: i32) -> Option<Self> {
        match value {
            COPYTUPLE => Some(Self::CopyTuple),
            INTERPOLATE => Some(Self::Interpolate),
            PASSDATA => Some(Self::PassData),
            ALLCOPY => Some(Self::AllCopy),
            _ => None,
        }
    }

    const fn index(self) -> Option<usize> {
        match self {
            Self::CopyTuple => Some(0),
            Self::Interpolate => Some(1),
            Self::PassData => Some(2),
            Self::AllCopy => None,
        }
    }
}

impl DataSetAttribute {
    pub const ALL: [Self; 12] = [
        Self::Scalars,
        Self::Vectors,
        Self::Normals,
        Self::TCoords,
        Self::Tensors,
        Self::GlobalIds,
        Self::PedigreeIds,
        Self::EdgeFlag,
        Self::Tangents,
        Self::RationalWeights,
        Self::HigherOrderDegrees,
        Self::ProcessIds,
    ];

    /// VTK: `vtkDataSetAttributes::GetAttributeTypeAsString`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Scalars => "Scalars",
            Self::Vectors => "Vectors",
            Self::Normals => "Normals",
            Self::TCoords => "TCoords",
            Self::Tensors => "Tensors",
            Self::GlobalIds => "GlobalIds",
            Self::PedigreeIds => "PedigreeIds",
            Self::EdgeFlag => "EdgeFlag",
            Self::Tangents => "Tangents",
            Self::RationalWeights => "RationalWeights",
            Self::HigherOrderDegrees => "HigherOrderDegrees",
            Self::ProcessIds => "ProcessIds",
        }
    }

    /// VTK: `vtkDataSetAttributes::GetLongAttributeTypeAsString`.
    pub(crate) fn long_name(self) -> &'static str {
        match self {
            Self::Scalars => "vtkDataSetAttributes::SCALARS",
            Self::Vectors => "vtkDataSetAttributes::VECTORS",
            Self::Normals => "vtkDataSetAttributes::NORMALS",
            Self::TCoords => "vtkDataSetAttributes::TCOORDS",
            Self::Tensors => "vtkDataSetAttributes::TENSORS",
            Self::GlobalIds => "vtkDataSetAttributes::GLOBALIDS",
            Self::PedigreeIds => "vtkDataSetAttributes::PEDIGREEIDS",
            Self::EdgeFlag => "vtkDataSetAttributes::EDGEFLAG",
            Self::Tangents => "vtkDataSetAttributes::TANGENTS",
            Self::RationalWeights => "vtkDataSetAttributes::RATIONALWEIGHTS",
            Self::HigherOrderDegrees => "vtkDataSetAttributes::HIGHERORDERDEGREES",
            Self::ProcessIds => "vtkDataSetAttributes::PROCESSIDS",
        }
    }

    fn component_rule(self) -> ComponentRule {
        match self {
            Self::Scalars => ComponentRule::NoLimit,
            Self::Vectors | Self::Normals | Self::Tangents | Self::HigherOrderDegrees => {
                ComponentRule::Exact(3)
            }
            Self::TCoords => ComponentRule::Max(3),
            Self::Tensors => ComponentRule::Tensor,
            Self::GlobalIds
            | Self::PedigreeIds
            | Self::EdgeFlag
            | Self::RationalWeights
            | Self::ProcessIds => ComponentRule::Exact(1),
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Scalars => 0,
            Self::Vectors => 1,
            Self::Normals => 2,
            Self::TCoords => 3,
            Self::Tensors => 4,
            Self::GlobalIds => 5,
            Self::PedigreeIds => 6,
            Self::EdgeFlag => 7,
            Self::Tangents => 8,
            Self::RationalWeights => 9,
            Self::HigherOrderDegrees => 10,
            Self::ProcessIds => 11,
        }
    }

    pub(crate) fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    pub(crate) fn from_i32(index: i32) -> Option<Self> {
        if index < 0 {
            return None;
        }
        Self::from_index(index as usize)
    }
}

pub const SCALARS: i32 = 0;
pub const VECTORS: i32 = 1;
pub const NORMALS: i32 = 2;
pub const TCOORDS: i32 = 3;
pub const TENSORS: i32 = 4;
pub const GLOBALIDS: i32 = 5;
pub const PEDIGREEIDS: i32 = 6;
pub const EDGEFLAG: i32 = 7;
pub const TANGENTS: i32 = 8;
pub const RATIONALWEIGHTS: i32 = 9;
pub const HIGHERORDERDEGREES: i32 = 10;
pub const PROCESSIDS: i32 = 11;
pub const NUM_ATTRIBUTES: i32 = 12;

pub const COPYTUPLE: i32 = 0;
pub const INTERPOLATE: i32 = 1;
pub const PASSDATA: i32 = 2;
pub const ALLCOPY: i32 = 3;

pub const DUPLICATECELL: u8 = 1;
pub const HIGHCONNECTIVITYCELL: u8 = 2;
pub const LOWCONNECTIVITYCELL: u8 = 4;
pub const REFINEDCELL: u8 = 8;
pub const EXTERIORCELL: u8 = 16;
pub const HIDDENCELL: u8 = 32;

pub const DUPLICATEPOINT: u8 = 1;
pub const HIDDENPOINT: u8 = 2;

/// Errors returned by compact `vtkDataSetAttributes` helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSetAttributesError {
    MissingArray(String),
    TupleComponentMismatch {
        from_components: usize,
        to_components: usize,
    },
    TupleOutOfRange {
        array: String,
        tuple: usize,
    },
    TupleIdLengthMismatch {
        from_len: usize,
        to_len: usize,
    },
}

impl std::fmt::Display for DataSetAttributesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingArray(name) => write!(f, "missing array '{name}'"),
            Self::TupleComponentMismatch {
                from_components,
                to_components,
            } => write!(
                f,
                "tuple component mismatch: source has {from_components}, target has {to_components}"
            ),
            Self::TupleOutOfRange { array, tuple } => {
                write!(f, "tuple {tuple} is out of range for array '{array}'")
            }
            Self::TupleIdLengthMismatch { from_len, to_len } => write!(
                f,
                "tuple id list length mismatch: source has {from_len}, target has {to_len}"
            ),
        }
    }
}

impl std::error::Error for DataSetAttributesError {}

/// Shared storage for data arrays and active attribute names.
#[derive(Debug, Clone, PartialEq)]
struct DataSetAttributesStorage {
    field_data: FieldData,
    modified_time: u64,
    copy_attribute_flags: [[i32; DataSetAttribute::ALL.len()]; 3],
    required_arrays: Vec<usize>,
    target_indices: Vec<isize>,
    active_scalars: Option<String>,
    active_vectors: Option<String>,
    active_normals: Option<String>,
    active_tcoords: Option<String>,
    active_tensors: Option<String>,
    active_global_ids: Option<String>,
    active_pedigree_ids: Option<String>,
    active_edge_flag: Option<String>,
    active_tangents: Option<String>,
    active_rational_weights: Option<String>,
    active_higher_order_degrees: Option<String>,
    active_process_ids: Option<String>,
}

impl Default for DataSetAttributesStorage {
    fn default() -> Self {
        let mut copy_attribute_flags = [[1; DataSetAttribute::ALL.len()]; 3];
        copy_attribute_flags[DataSetAttributeCopyOperation::CopyTuple.index().unwrap()]
            [DataSetAttribute::GlobalIds.index()] = 0;
        copy_attribute_flags[DataSetAttributeCopyOperation::Interpolate.index().unwrap()]
            [DataSetAttribute::GlobalIds.index()] = 0;
        copy_attribute_flags[DataSetAttributeCopyOperation::Interpolate.index().unwrap()]
            [DataSetAttribute::PedigreeIds.index()] = 0;
        copy_attribute_flags[DataSetAttributeCopyOperation::Interpolate.index().unwrap()]
            [DataSetAttribute::ProcessIds.index()] = 0;

        Self {
            field_data: FieldData::new(),
            modified_time: 0,
            copy_attribute_flags,
            required_arrays: Vec::new(),
            target_indices: Vec::new(),
            active_scalars: None,
            active_vectors: None,
            active_normals: None,
            active_tcoords: None,
            active_tensors: None,
            active_global_ids: None,
            active_pedigree_ids: None,
            active_edge_flag: None,
            active_tangents: None,
            active_rational_weights: None,
            active_higher_order_degrees: None,
            active_process_ids: None,
        }
    }
}

/// Field data plus VTK's active attribute designations.
///
/// VTK origin: `VTK/Common/DataModel/vtkDataSetAttributes.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct DataSetAttributes {
    storage: Arc<DataSetAttributesStorage>,
}

impl DataSetAttributes {
    /// VTK: `vtkDataSetAttributes::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(DataSetAttributesStorage::default()),
        }
    }

    /// VTK: `vtkDataSetAttributes::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    pub(crate) fn field_data(&self) -> &FieldData {
        &self.storage.field_data
    }

    #[cfg(test)]
    pub(crate) fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.storage_mut().field_data
    }

    /// VTK: `vtkDataSetAttributes::Initialize`.
    pub fn initialize(&mut self) {
        let mut field_data = self.storage.field_data.clone();
        field_data.initialize();
        let mut storage = DataSetAttributesStorage::default();
        storage.field_data = field_data;
        self.storage = Arc::new(storage);
    }

    /// VTK: `vtkDataSetAttributes::GetAttributeTypeAsString`.
    pub fn get_attribute_type_as_string(attribute_type: i32) -> Option<&'static str> {
        DataSetAttribute::from_i32(attribute_type).map(Self::attribute_type_as_string)
    }

    pub(crate) fn attribute_type_as_string(role: DataSetAttribute) -> &'static str {
        role.as_str()
    }

    /// VTK: `vtkDataSetAttributes::GetLongAttributeTypeAsString`.
    pub fn get_long_attribute_type_as_string(attribute_type: i32) -> Option<&'static str> {
        DataSetAttribute::from_i32(attribute_type).map(Self::long_attribute_type_as_string)
    }

    pub(crate) fn long_attribute_type_as_string(role: DataSetAttribute) -> &'static str {
        role.long_name()
    }

    /// VTK: `vtkFieldData::AddArray`.
    pub fn add_array(&mut self, array: AnyArray) -> i32 {
        self.storage_mut().field_data.add_array(array)
    }

    pub(crate) fn add_field_data_array(&mut self, array: FieldDataArray) -> usize {
        self.storage_mut().field_data.add_field_data_array(array)
    }

    /// VTK: `vtkFieldData::RemoveArray(const char*)`.
    pub fn remove_array(&mut self, name: &str) {
        self.remove_field_data_array(name);
    }

    pub(crate) fn remove_field_data_array(&mut self, name: &str) -> Option<FieldDataArray> {
        let removed = self.storage_mut().field_data.remove_field_data_array(name);
        if removed.is_some() {
            self.clear_active_name(name);
        }
        removed
    }

    /// VTK: `vtkDataSetAttributes::RemoveArray(int)`.
    pub fn remove_array_by_index(&mut self, index: i32) {
        self.remove_field_data_array_by_index_i32(index);
    }

    fn remove_field_data_array_by_index_i32(&mut self, index: i32) -> Option<FieldDataArray> {
        let index = usize::try_from(index).ok()?;
        self.remove_field_data_array_by_index(index)
    }

    pub(crate) fn remove_field_data_array_by_index(
        &mut self,
        index: usize,
    ) -> Option<FieldDataArray> {
        let removed = self
            .storage_mut()
            .field_data
            .remove_field_data_array_by_index(index);
        if let Some(array) = removed.as_ref() {
            self.clear_active_name(array.get_name());
        }
        removed
    }

    /// VTK: `vtkFieldData::GetNumberOfArrays`.
    pub fn get_number_of_arrays(&self) -> i32 {
        self.storage.field_data.get_number_of_arrays()
    }

    /// VTK: `vtkFieldData::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.storage.field_data.get_number_of_components()
    }

    /// VTK: `vtkFieldData::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        self.storage.field_data.get_number_of_tuples()
    }

    /// VTK: `vtkDataSetAttributes::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.storage.field_data.print_self()
    }

    /// VTK: `vtkFieldData::Allocate`.
    pub fn allocate(&mut self, values_per_array: VtkIdType, ext: VtkIdType) -> bool {
        self.storage_mut()
            .field_data
            .allocate(values_per_array, ext)
    }

    /// VTK: `vtkFieldData::AllocateArrays`.
    pub fn allocate_arrays(&mut self, number_of_arrays: i32) {
        self.storage_mut()
            .field_data
            .allocate_arrays(number_of_arrays);
        self.sync_active_after_field_data_change();
    }

    /// VTK: `vtkFieldData::HasArray`.
    pub fn has_array(&self, name: &str) -> bool {
        self.storage.field_data.has_array(name)
    }

    /// VTK: `vtkFieldData::GetArray(const char*)`.
    pub fn get_array(&self, name: &str) -> Option<&AnyArray> {
        self.storage.field_data.get_array(name)
    }

    /// VTK: `vtkFieldData::GetArray(const char*, int&)`.
    pub fn get_array_with_index(&self, name: &str, index: &mut i32) -> Option<&AnyArray> {
        self.storage.field_data.get_array_with_index(name, index)
    }

    /// VTK: `vtkFieldData::GetArray(int)`.
    pub fn get_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.storage.field_data.get_array_by_index(index)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*)`.
    pub fn get_abstract_array(&self, name: &str) -> Option<&AnyArray> {
        self.storage.field_data.get_abstract_array(name)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*, int&)`.
    pub fn get_abstract_array_with_index(&self, name: &str, index: &mut i32) -> Option<&AnyArray> {
        self.storage
            .field_data
            .get_abstract_array_with_index(name, index)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(int)`.
    pub fn get_abstract_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.storage.field_data.get_abstract_array_by_index(index)
    }

    /// VTK: `vtkFieldData::GetArrayName`.
    pub fn get_array_name(&self, index: i32) -> Option<&str> {
        self.storage.field_data.get_array_name(index)
    }

    /// VTK: `vtkFieldData::GetGhostsToSkip`.
    pub fn get_ghosts_to_skip(&self) -> u8 {
        self.storage.field_data.get_ghosts_to_skip()
    }

    /// VTK: `vtkFieldData::SetGhostsToSkip`.
    pub fn set_ghosts_to_skip(&mut self, ghosts_to_skip: u8) {
        self.storage_mut()
            .field_data
            .set_ghosts_to_skip(ghosts_to_skip);
    }

    /// VTK: `vtkFieldData::GetGhostArray`.
    pub fn get_ghost_array(&self) -> Option<&AnyArray> {
        self.storage.field_data.get_ghost_array()
    }

    /// VTK: `vtkFieldData::HasAnyGhostBitSet`.
    pub fn has_any_ghost_bit_set(&self, bit_flag: i32) -> bool {
        self.storage.field_data.has_any_ghost_bit_set(bit_flag)
    }

    pub(crate) fn allocate_ghost_array(&mut self, number_of_values: VtkIdType) {
        if self.get_ghost_array().is_some() {
            return;
        }

        let number_of_values = number_of_values.max(0);
        let mut ghosts =
            AnyArray::create_array(VtkDataType::UnsignedChar).expect("unsigned char array");
        ghosts.set_name(FieldData::ghost_array_name());
        ghosts.set_number_of_components(1);
        ghosts.set_number_of_tuples(number_of_values);
        for tuple_idx in 0..number_of_values as usize {
            let _ = ghosts.insert_numeric_tuple_from_f64_checked(tuple_idx, &[0.0]);
        }
        self.add_array(ghosts);
    }

    pub(crate) fn set_ghost_bit(&mut self, tuple_id: VtkIdType, bit: u8, enabled: bool) -> bool {
        if tuple_id < 0 {
            return false;
        }
        let tuple_idx = vtk_id_to_usize(tuple_id);
        let Some(array) = self.get_array_mut(FieldData::ghost_array_name()) else {
            return false;
        };
        if !matches!(array.get_data(), AnyArray::UnsignedChar(_)) {
            return false;
        }
        let Ok(tuple) = array.get_data().numeric_tuple_as_f64_checked(tuple_idx) else {
            return false;
        };
        let Some(value) = tuple.first() else {
            return false;
        };
        let mut value = *value as u8;
        if enabled {
            value |= bit;
        } else {
            value &= !bit;
        }
        array
            .get_data_mut()
            .insert_numeric_tuple_from_f64_checked(tuple_idx, &[f64::from(value)])
            .is_ok()
    }

    pub(crate) fn get_field_data_array(&self, name: &str) -> Option<&FieldDataArray> {
        self.storage.field_data.get_field_data_array(name)
    }

    pub(crate) fn get_array_mut(&mut self, name: &str) -> Option<&mut FieldDataArray> {
        self.storage_mut().field_data.get_array_mut(name)
    }

    pub(crate) fn get_field_data_array_by_index(&self, index: usize) -> Option<&FieldDataArray> {
        self.storage.field_data.get_field_data_array_by_index(index)
    }

    pub(crate) fn get_array_by_index_mut(&mut self, index: usize) -> Option<&mut FieldDataArray> {
        self.storage_mut().field_data.arrays_mut().get_mut(index)
    }

    #[cfg(test)]
    pub(crate) fn array_names(&self) -> Vec<&str> {
        self.storage.field_data.names()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FieldDataArray> {
        self.storage.field_data.iter()
    }

    pub fn squeeze(&mut self) {
        for array in self.storage_mut().field_data.arrays_mut() {
            array.squeeze();
        }
    }

    pub fn get_actual_memory_size(&self) -> usize {
        self.storage
            .field_data
            .iter()
            .map(FieldDataArray::get_actual_memory_size)
            .sum()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.field_data.get_m_time())
    }

    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let number_of_tuples = vtk_id_to_usize(number_of_tuples);
        for array in self.storage_mut().field_data.arrays_mut() {
            array.set_number_of_tuples(number_of_tuples);
        }
    }

    pub(crate) fn remove_tuple_swap_with_last(&mut self, tuple: usize) {
        for array in self.storage_mut().field_data.arrays_mut() {
            array.remove_tuple_swap_with_last(tuple);
        }
    }

    /// VTK: `vtkDataSetAttributes::SetCopyAttribute`.
    pub fn set_copy_attribute(&mut self, index: i32, value: i32, ctype: i32) {
        let Some(role) = DataSetAttribute::from_i32(index) else {
            return;
        };
        let Some(operation) = DataSetAttributeCopyOperation::from_i32(ctype) else {
            return;
        };
        self.set_copy_attribute_role(role, value, operation);
    }

    pub(crate) fn set_copy_attribute_role(
        &mut self,
        role: DataSetAttribute,
        value: i32,
        operation: DataSetAttributeCopyOperation,
    ) {
        match operation.index() {
            Some(operation_index) => {
                self.storage_mut().copy_attribute_flags[operation_index][role.index()] = value;
            }
            None => {
                for operation in DataSetAttributeCopyOperation::SINGLE {
                    self.set_copy_attribute_role(role, value, operation);
                }
            }
        }
    }

    /// VTK: `vtkDataSetAttributes::GetCopyAttribute`.
    pub fn get_copy_attribute(&self, index: i32, ctype: i32) -> i32 {
        let Some(role) = DataSetAttribute::from_i32(index) else {
            return -1;
        };
        let Some(operation) = DataSetAttributeCopyOperation::from_i32(ctype) else {
            return -1;
        };
        self.get_copy_attribute_role(role, operation)
    }

    pub(crate) fn get_copy_attribute_role(
        &self,
        role: DataSetAttribute,
        operation: DataSetAttributeCopyOperation,
    ) -> i32 {
        match operation.index() {
            Some(operation_index) => {
                self.storage.copy_attribute_flags[operation_index][role.index()]
            }
            None => DataSetAttributeCopyOperation::SINGLE
                .into_iter()
                .all(|operation| self.get_copy_attribute_role(role, operation) != 0)
                .into(),
        }
    }

    pub(crate) fn copy_attribute_role_enabled(
        &self,
        role: DataSetAttribute,
        operation: DataSetAttributeCopyOperation,
    ) -> bool {
        self.get_copy_attribute_role(role, operation) != 0
    }

    /// VTK: `vtkDataSetAttributes::CopyAllOn`.
    pub fn copy_all_on(&mut self, ctype: i32) {
        let Some(operation) = DataSetAttributeCopyOperation::from_i32(ctype) else {
            return;
        };
        for role in DataSetAttribute::ALL {
            self.set_copy_attribute_role(role, 1, operation);
        }
    }

    /// VTK: `vtkDataSetAttributes::CopyAllOff`.
    pub fn copy_all_off(&mut self, ctype: i32) {
        let Some(operation) = DataSetAttributeCopyOperation::from_i32(ctype) else {
            return;
        };
        for role in DataSetAttribute::ALL {
            self.set_copy_attribute_role(role, 0, operation);
        }
    }

    pub fn set_copy_scalars(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(SCALARS, i32::from(value), ctype);
    }

    pub fn get_copy_scalars(&self, ctype: i32) -> bool {
        self.get_copy_attribute(SCALARS, ctype) != 0
    }

    pub fn set_copy_vectors(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(VECTORS, i32::from(value), ctype);
    }

    pub fn get_copy_vectors(&self, ctype: i32) -> bool {
        self.get_copy_attribute(VECTORS, ctype) != 0
    }

    pub fn set_copy_normals(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(NORMALS, i32::from(value), ctype);
    }

    pub fn get_copy_normals(&self, ctype: i32) -> bool {
        self.get_copy_attribute(NORMALS, ctype) != 0
    }

    pub fn set_copy_tcoords(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(TCOORDS, i32::from(value), ctype);
    }

    pub fn get_copy_tcoords(&self, ctype: i32) -> bool {
        self.get_copy_attribute(TCOORDS, ctype) != 0
    }

    pub fn set_copy_tensors(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(TENSORS, i32::from(value), ctype);
    }

    pub fn get_copy_tensors(&self, ctype: i32) -> bool {
        self.get_copy_attribute(TENSORS, ctype) != 0
    }

    pub fn set_copy_global_ids(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(GLOBALIDS, i32::from(value), ctype);
    }

    pub fn get_copy_global_ids(&self, ctype: i32) -> bool {
        self.get_copy_attribute(GLOBALIDS, ctype) != 0
    }

    pub fn set_copy_pedigree_ids(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(PEDIGREEIDS, i32::from(value), ctype);
    }

    pub fn get_copy_pedigree_ids(&self, ctype: i32) -> bool {
        self.get_copy_attribute(PEDIGREEIDS, ctype) != 0
    }

    pub fn set_copy_tangents(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(TANGENTS, i32::from(value), ctype);
    }

    pub fn get_copy_tangents(&self, ctype: i32) -> bool {
        self.get_copy_attribute(TANGENTS, ctype) != 0
    }

    pub fn set_copy_rational_weights(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(RATIONALWEIGHTS, i32::from(value), ctype);
    }

    pub fn get_copy_rational_weights(&self, ctype: i32) -> bool {
        self.get_copy_attribute(RATIONALWEIGHTS, ctype) != 0
    }

    pub fn set_copy_higher_order_degrees(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(HIGHERORDERDEGREES, i32::from(value), ctype);
    }

    pub fn get_copy_higher_order_degrees(&self, ctype: i32) -> bool {
        self.get_copy_attribute(HIGHERORDERDEGREES, ctype) != 0
    }

    pub fn set_copy_process_ids(&mut self, value: bool, ctype: i32) {
        self.set_copy_attribute(PROCESSIDS, i32::from(value), ctype);
    }

    pub fn get_copy_process_ids(&self, ctype: i32) -> bool {
        self.get_copy_attribute(PROCESSIDS, ctype) != 0
    }

    /// VTK: `vtkDataSetAttributes::PassData`.
    pub fn pass_data(&mut self, source: &Self) {
        let required_names = self.required_pass_data_array_names(source);

        for role in DataSetAttribute::ALL {
            if self.copy_attribute_role_enabled(role, DataSetAttributeCopyOperation::PassData) {
                if let Some(old_name) = self.active_name(role).map(str::to_string) {
                    self.remove_field_data_array(&old_name);
                } else {
                    self.set_active_name(role, None);
                }
            }
        }

        for name in &required_names {
            if let Some(source_array) = source.get_field_data_array(name) {
                self.add_field_data_array(source_array.shallow_clone());
            }
        }

        for role in DataSetAttribute::ALL {
            if !self.copy_attribute_role_enabled(role, DataSetAttributeCopyOperation::PassData) {
                continue;
            }
            let Some(name) = source.active_name(role) else {
                continue;
            };
            if required_names.iter().any(|required| required == name) {
                let index = self
                    .storage
                    .field_data
                    .find_array_index(name)
                    .and_then(|index| i32::try_from(index).ok())
                    .unwrap_or(-1);
                let _ = self.set_active_attribute_by_index_role(role, index);
            }
        }
    }

    /// VTK: `vtkDataSetAttributes::CopyAllocate(vtkDataSetAttributesFieldList*)`.
    pub fn copy_allocate(
        &mut self,
        field_list: &mut DataSetAttributesFieldList,
        size: usize,
        ext: usize,
    ) {
        field_list.copy_allocate(self, DataSetAttributeCopyOperation::CopyTuple, size, ext);
    }

    /// VTK: `vtkDataSetAttributes::CopyAllocate(vtkDataSetAttributes*, vtkIdType, vtkIdType, int)`.
    pub fn copy_allocate_from(
        &mut self,
        source: &Self,
        size: VtkIdType,
        ext: VtkIdType,
        shallow_copy_arrays: i32,
    ) {
        self.internal_copy_allocate(
            source,
            DataSetAttributeCopyOperation::CopyTuple,
            size,
            ext,
            shallow_copy_arrays,
            true,
        );
    }

    /// VTK: `vtkDataSetAttributes::InterpolateAllocate(vtkDataSetAttributesFieldList*)`.
    pub fn interpolate_allocate(
        &mut self,
        field_list: &mut DataSetAttributesFieldList,
        size: usize,
        ext: usize,
    ) {
        field_list.copy_allocate(self, DataSetAttributeCopyOperation::Interpolate, size, ext);
    }

    /// VTK: `vtkDataSetAttributes::InterpolateAllocate(vtkDataSetAttributes*, vtkIdType, vtkIdType, int)`.
    pub fn interpolate_allocate_from(
        &mut self,
        source: &Self,
        size: VtkIdType,
        ext: VtkIdType,
        shallow_copy_arrays: i32,
    ) {
        self.internal_copy_allocate(
            source,
            DataSetAttributeCopyOperation::Interpolate,
            size,
            ext,
            shallow_copy_arrays,
            true,
        );
    }

    /// VTK: `vtkDataSetAttributes::SetupForCopy`.
    pub fn setup_for_copy(&mut self, source: &Self) {
        self.internal_copy_allocate(
            source,
            DataSetAttributeCopyOperation::CopyTuple,
            0,
            0,
            0,
            false,
        );
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributesFieldList*)`.
    pub fn copy_data(
        &mut self,
        field_list: &DataSetAttributesFieldList,
        input_index: usize,
        input: &Self,
        from_id: usize,
        to_id: usize,
    ) -> Result<(), DataSetAttributesError> {
        field_list.copy_data(input_index, input, from_id, self, to_id)
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributesFieldList*, vtkIdType, vtkIdType, vtkIdType)`.
    pub fn copy_data_range(
        &mut self,
        field_list: &DataSetAttributesFieldList,
        input_index: usize,
        input: &Self,
        dst_start: VtkIdType,
        n: VtkIdType,
        src_start: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        field_list.copy_data_range(
            input_index,
            input,
            vtk_id_to_usize(src_start),
            vtk_id_to_usize(n),
            self,
            vtk_id_to_usize(dst_start),
        )
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdType, vtkIdType)`.
    pub fn copy_data_from(
        &mut self,
        from_pd: &Self,
        from_id: VtkIdType,
        to_id: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        let from_id = vtk_id_to_usize(from_id);
        let to_id = vtk_id_to_usize(to_id);
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;
            Self::copy_tuple(from_array, to_array, from_id, to_id)?;
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdList*, vtkIdList*)`.
    pub fn copy_data_from_ids(
        &mut self,
        from_pd: &Self,
        from_ids: &[VtkIdType],
        to_ids: &[VtkIdType],
    ) -> Result<(), DataSetAttributesError> {
        if from_ids.len() != to_ids.len() {
            return Err(DataSetAttributesError::TupleIdLengthMismatch {
                from_len: from_ids.len(),
                to_len: to_ids.len(),
            });
        }
        if to_ids.is_empty() {
            return Ok(());
        }

        let from_ids: Vec<_> = from_ids.iter().copied().map(vtk_id_to_usize).collect();
        let to_ids: Vec<_> = to_ids.iter().copied().map(vtk_id_to_usize).collect();
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;
            for (&from_id, &to_id) in from_ids.iter().zip(&to_ids) {
                Self::copy_tuple(from_array, to_array, from_id, to_id)?;
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdList*, vtkIdType)`.
    pub fn copy_data_from_ids_to_start(
        &mut self,
        from_pd: &Self,
        from_ids: &[VtkIdType],
        dest_start: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        if from_ids.is_empty() {
            return Ok(());
        }

        let dest_start = vtk_id_to_usize(dest_start);
        let from_ids: Vec<_> = from_ids.iter().copied().map(vtk_id_to_usize).collect();
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;
            for (offset, &from_id) in from_ids.iter().enumerate() {
                Self::copy_tuple(from_array, to_array, from_id, dest_start + offset)?;
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdType, vtkIdType, vtkIdType)`.
    pub fn copy_data_range_from(
        &mut self,
        from_pd: &Self,
        dst_start: VtkIdType,
        n: VtkIdType,
        src_start: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        let n = vtk_id_to_usize(n);
        if n == 0 {
            return Ok(());
        }

        let dst_start = vtk_id_to_usize(dst_start);
        let src_start = vtk_id_to_usize(src_start);
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;
            for offset in 0..n {
                Self::copy_tuple(from_array, to_array, src_start + offset, dst_start + offset)?;
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::CopyStructuredData`.
    pub fn copy_structured_data(
        &mut self,
        from_pd: &Self,
        in_ext: [i32; 6],
        out_ext: [i32; 6],
        set_size: bool,
    ) -> Result<(), DataSetAttributesError> {
        let input_tuples = Self::structured_extent_tuple_count(in_ext);
        let output_tuples = Self::structured_extent_tuple_count(out_ext);
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            if from_array.get_number_of_tuples() as usize != input_tuples {
                continue;
            }

            let is_ghost_array = from_array.get_name() == FieldData::ghost_array_name();
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;
            if set_size && to_array.get_number_of_tuples() as usize != output_tuples {
                to_array.set_number_of_tuples(output_tuples);
                if is_ghost_array {
                    Self::fill_unsigned_char_component(to_array, 0xff);
                }
            }

            if !Self::structured_copy_supported(from_array.get_data(), to_array.get_data()) {
                continue;
            }

            for z_idx in out_ext[4]..=out_ext[5] {
                for y_idx in out_ext[2]..=out_ext[3] {
                    for x_idx in out_ext[0]..=out_ext[1] {
                        let from_id = Self::structured_extent_tuple_id(in_ext, x_idx, y_idx, z_idx);
                        let to_id = Self::structured_extent_tuple_id(out_ext, x_idx, y_idx, z_idx);
                        if is_ghost_array
                            && Self::merge_unsigned_char_ghost_tuple(
                                from_array, to_array, from_id, to_id,
                            )?
                        {
                            continue;
                        }
                        Self::copy_tuple(from_array, to_array, from_id, to_id)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::InterpolatePoint(vtkDataSetAttributesFieldList*)`.
    pub fn interpolate_point(
        &mut self,
        field_list: &DataSetAttributesFieldList,
        input_index: usize,
        input: &Self,
        to_id: usize,
        input_ids: &[usize],
        weights: &[f64],
    ) -> Result<(), DataSetAttributesError> {
        field_list.interpolate_point(input_index, input, input_ids, weights, self, to_id)
    }

    /// VTK: `vtkDataSetAttributes::InterpolatePoint(vtkDataSetAttributes*, vtkIdType, vtkIdList*, double*)`.
    pub fn interpolate_point_from(
        &mut self,
        from_pd: &Self,
        to_id: VtkIdType,
        pt_ids: &[VtkIdType],
        weights: &[f64],
    ) -> Result<(), DataSetAttributesError> {
        if pt_ids.len() != weights.len() {
            return Err(DataSetAttributesError::TupleIdLengthMismatch {
                from_len: pt_ids.len(),
                to_len: weights.len(),
            });
        }

        let to_id = vtk_id_to_usize(to_id);
        let pt_ids: Vec<_> = pt_ids.iter().copied().map(vtk_id_to_usize).collect();
        let copy_pairs = self.copy_target_pairs();
        let nearest_weighted_tuple = |input_ids: &[usize], weights: &[f64]| -> Option<usize> {
            let mut nearest = *input_ids.first()?;
            let mut max_weight = 0.0;
            for (&input_id, &weight) in input_ids.iter().zip(weights) {
                if weight > max_weight {
                    max_weight = weight;
                    nearest = input_id;
                }
            }
            Some(nearest)
        };

        for (source_index, target_index) in copy_pairs {
            let nearest_neighbor =
                DataSetAttribute::from_i32(self.is_array_an_attribute(target_index as i32))
                    .map(|role| {
                        self.get_copy_attribute_role(
                            role,
                            DataSetAttributeCopyOperation::Interpolate,
                        ) == 2
                    })
                    .unwrap_or(false);
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;

            let copied = if nearest_neighbor {
                let source_id = nearest_weighted_tuple(&pt_ids, weights).ok_or_else(|| {
                    DataSetAttributesError::TupleOutOfRange {
                        array: from_array.get_name().to_string(),
                        tuple: 0,
                    }
                })?;
                to_array.copy_tuple_from(from_array, source_id, to_id)
            } else {
                to_array.interpolate_tuple_from(from_array, &pt_ids, weights, to_id)
            };
            if !copied {
                return Err(DataSetAttributesError::TupleOutOfRange {
                    array: to_array.get_name().to_string(),
                    tuple: pt_ids.iter().copied().max().unwrap_or(to_id),
                });
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::InterpolateEdge`.
    pub fn interpolate_edge(
        &mut self,
        from_pd: &Self,
        to_id: VtkIdType,
        p1: VtkIdType,
        p2: VtkIdType,
        t: f64,
    ) -> Result<(), DataSetAttributesError> {
        let to_id = vtk_id_to_usize(to_id);
        let p1 = vtk_id_to_usize(p1);
        let p2 = vtk_id_to_usize(p2);
        let copy_pairs = self.copy_target_pairs();

        for (source_index, target_index) in copy_pairs {
            let nearest_neighbor =
                DataSetAttribute::from_i32(self.is_array_an_attribute(target_index as i32))
                    .map(|role| {
                        self.get_copy_attribute_role(
                            role,
                            DataSetAttributeCopyOperation::Interpolate,
                        ) == 2
                    })
                    .unwrap_or(false);
            let from_array = from_pd
                .get_field_data_array_by_index(source_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(source_index.to_string()))?;
            let to_array = self
                .get_array_by_index_mut(target_index)
                .ok_or_else(|| DataSetAttributesError::MissingArray(target_index.to_string()))?;

            let copied = if nearest_neighbor {
                let source_id = if t < 0.5 { p1 } else { p2 };
                to_array.copy_tuple_from(from_array, source_id, to_id)
            } else {
                to_array.interpolate_tuple_from(from_array, &[p1, p2], &[1.0 - t, t], to_id)
            };
            if !copied {
                return Err(DataSetAttributesError::TupleOutOfRange {
                    array: to_array.get_name().to_string(),
                    tuple: to_id,
                });
            }
        }

        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::InterpolateTime`.
    pub fn interpolate_time(
        &mut self,
        from1: &Self,
        from2: &Self,
        id: VtkIdType,
        t: f64,
    ) -> Result<(), DataSetAttributesError> {
        let id = vtk_id_to_usize(id);
        let id_vtk = id as VtkIdType;

        for role in DataSetAttribute::ALL {
            let flag =
                self.get_copy_attribute_role(role, DataSetAttributeCopyOperation::Interpolate);
            if flag == 0 {
                continue;
            }

            let (Some(from_array1), Some(from_array2)) = (
                from1.get_field_data_attribute(role).cloned(),
                from2.get_field_data_attribute(role).cloned(),
            ) else {
                continue;
            };

            let role_name =
                Self::get_attribute_type_as_string(role.index() as i32).unwrap_or("UNKNOWN");
            let to_array = self
                .get_attribute_mut(role)
                .ok_or_else(|| DataSetAttributesError::MissingArray(role_name.to_string()))?;

            let copied = if flag == 2 {
                if t < 0.5 {
                    to_array.copy_tuple_from(&from_array1, id, id)
                } else {
                    to_array.copy_tuple_from(&from_array2, id, id)
                }
            } else {
                to_array.get_data_mut().interpolate_tuple_between(
                    id_vtk,
                    id_vtk,
                    from_array1.get_data(),
                    id_vtk,
                    from_array2.get_data(),
                    t,
                )
            };

            if !copied {
                return Err(DataSetAttributesError::TupleOutOfRange {
                    array: to_array.get_name().to_string(),
                    tuple: id,
                });
            }
        }

        Ok(())
    }

    fn internal_copy_allocate(
        &mut self,
        source: &Self,
        operation: DataSetAttributeCopyOperation,
        size: VtkIdType,
        _ext: VtkIdType,
        shallow_copy_arrays: i32,
        create_new_arrays: bool,
    ) {
        let required_arrays = self.required_array_indices(source, operation);
        if required_arrays.is_empty() {
            self.storage_mut().required_arrays = required_arrays;
            return;
        }

        let mut target_indices = vec![-1; source.get_number_of_arrays().max(0) as usize];
        let allocation_size = if size > 0 {
            size
        } else {
            source.get_number_of_tuples()
        };

        if create_new_arrays {
            let mut additions = Vec::new();
            for &source_index in &required_arrays {
                let Some(source_array) = source.get_field_data_array_by_index(source_index) else {
                    continue;
                };
                let values = vtk_id_to_usize(allocation_size)
                    .saturating_mul(source_array.get_number_of_components());
                let target_array = if shallow_copy_arrays != 0 {
                    source_array.shallow_clone()
                } else {
                    let mut target_array = FieldDataArray::new_with_data_type(
                        source_array.get_name(),
                        source_array.get_number_of_components(),
                        source_array.get_data_type(),
                    );
                    target_array.reserve_values(values);
                    target_array
                };
                let attribute_role = source
                    .is_array_an_attribute(source_index as i32)
                    .try_into()
                    .ok()
                    .and_then(DataSetAttribute::from_index)
                    .filter(|&role| self.copy_attribute_role_enabled(role, operation));
                additions.push((source_index, target_array, attribute_role));
            }

            for (source_index, target_array, attribute_role) in additions {
                let target_index = self.add_field_data_array(target_array);
                target_indices[source_index] = target_index as isize;
                if let Some(role) = attribute_role {
                    let source_flag = source.get_copy_attribute_role(role, operation);
                    self.set_copy_attribute_role(role, source_flag, operation);
                    let _ = self.set_active_attribute_by_index_role(role, target_index as i32);
                }
            }
        } else {
            for &source_index in &required_arrays {
                if source_index < self.storage.field_data.get_number_of_arrays().max(0) as usize {
                    target_indices[source_index] = source_index as isize;
                }
            }
        }

        let storage = self.storage_mut();
        storage.required_arrays = required_arrays;
        storage.target_indices = target_indices;
    }

    fn structured_extent_tuple_count(ext: [i32; 6]) -> usize {
        let x = usize::try_from(ext[1] - ext[0] + 1).expect("structured extent x must be valid");
        let y = usize::try_from(ext[3] - ext[2] + 1).expect("structured extent y must be valid");
        let z = usize::try_from(ext[5] - ext[4] + 1).expect("structured extent z must be valid");
        x.saturating_mul(y).saturating_mul(z)
    }

    fn structured_extent_tuple_id(ext: [i32; 6], x: i32, y: i32, z: i32) -> usize {
        let x_dim =
            usize::try_from(ext[1] - ext[0] + 1).expect("structured extent x must be valid");
        let y_dim =
            usize::try_from(ext[3] - ext[2] + 1).expect("structured extent y must be valid");
        let x_offset =
            usize::try_from(x - ext[0]).expect("structured x coordinate must be in extent");
        let y_offset =
            usize::try_from(y - ext[2]).expect("structured y coordinate must be in extent");
        let z_offset =
            usize::try_from(z - ext[4]).expect("structured z coordinate must be in extent");
        x_offset + y_offset * x_dim + z_offset * x_dim * y_dim
    }

    fn structured_copy_supported(source: &AnyArray, target: &AnyArray) -> bool {
        match (source, target) {
            (AnyArray::String(_), AnyArray::String(_)) => true,
            (AnyArray::Variant(_), _) | (_, AnyArray::Variant(_)) => false,
            (source, target) => source.is_numeric() && target.is_numeric(),
        }
    }

    fn fill_unsigned_char_component(array: &mut FieldDataArray, value: u8) {
        if !matches!(array.get_data(), AnyArray::UnsignedChar(_))
            || array.get_number_of_components() != 1
        {
            return;
        }
        let tuple_count = array.get_number_of_tuples();
        for tuple in 0..tuple_count {
            let _ = array.get_data_mut().set_numeric_component_from_f64_checked(
                tuple,
                0,
                f64::from(value),
            );
        }
    }

    fn merge_unsigned_char_ghost_tuple(
        from_array: &FieldDataArray,
        to_array: &mut FieldDataArray,
        from_id: usize,
        to_id: usize,
    ) -> Result<bool, DataSetAttributesError> {
        if !matches!(from_array.get_data(), AnyArray::UnsignedChar(_))
            || !matches!(to_array.get_data(), AnyArray::UnsignedChar(_))
            || from_array.get_number_of_components() != 1
            || to_array.get_number_of_components() != 1
        {
            return Ok(false);
        }

        let source = from_array
            .get_data()
            .numeric_component_as_f64_checked(from_id, 0)
            .map_err(|_| DataSetAttributesError::TupleOutOfRange {
                array: from_array.get_name().to_string(),
                tuple: from_id,
            })? as u8;
        let target = to_array
            .get_data()
            .numeric_component_as_f64_checked(to_id, 0)
            .map_err(|_| DataSetAttributesError::TupleOutOfRange {
                array: to_array.get_name().to_string(),
                tuple: to_id,
            })? as u8;
        to_array
            .get_data_mut()
            .set_numeric_component_from_f64_checked(to_id, 0, f64::from(source & target))
            .map_err(|_| DataSetAttributesError::TupleOutOfRange {
                array: to_array.get_name().to_string(),
                tuple: to_id,
            })?;
        Ok(true)
    }

    fn copy_target_pairs(&self) -> Vec<(usize, usize)> {
        self.storage
            .required_arrays
            .iter()
            .filter_map(|&source_index| {
                self.storage
                    .target_indices
                    .get(source_index)
                    .and_then(|&target_index| usize::try_from(target_index).ok())
                    .map(|target_index| (source_index, target_index))
            })
            .collect()
    }

    fn required_array_indices(
        &self,
        source: &Self,
        operation: DataSetAttributeCopyOperation,
    ) -> Vec<usize> {
        let mut required = Vec::new();
        for (index, array) in source.iter().enumerate() {
            if self.storage.field_data.should_copy_array(array.get_name()) {
                if operation != DataSetAttributeCopyOperation::Interpolate
                    || array.get_data_type() != VtkDataType::IdType
                {
                    required.push(index);
                }
            }
        }

        for role in DataSetAttribute::ALL {
            let Some(name) = source.active_name(role) else {
                continue;
            };
            let Some(index) = source.storage.field_data.find_array_index(name) else {
                continue;
            };
            let Some(array) = source.get_field_data_array_by_index(index) else {
                continue;
            };
            let should_copy = self.copy_attribute_role_enabled(role, operation)
                && self.storage.field_data.should_copy_array(name)
                && source.get_field_data_attribute(role).is_some();

            if should_copy {
                if (operation != DataSetAttributeCopyOperation::Interpolate
                    || array.get_data_type() != VtkDataType::IdType)
                    && !required.contains(&index)
                {
                    required.push(index);
                }
            } else {
                required.retain(|&required| required != index);
            }
        }

        required
    }

    /// VTK: `vtkDataSetAttributes::ComputeRequiredArrays`.
    fn required_pass_data_array_names(&self, source: &Self) -> Vec<String> {
        let mut required = Vec::new();
        for array in source.iter() {
            if self.storage.field_data.should_copy_array(array.get_name()) {
                required.push(array.get_name().to_string());
            }
        }

        for role in DataSetAttribute::ALL {
            let Some(name) = source.active_name(role) else {
                continue;
            };
            if self.copy_attribute_role_enabled(role, DataSetAttributeCopyOperation::PassData)
                && self.storage.field_data.should_copy_array(name)
            {
                if !required.iter().any(|required| required == name) {
                    required.push(name.to_string());
                }
            } else {
                required.retain(|required| required != name);
            }
        }
        required
    }

    /// VTK: `vtkDataSetAttributes::SetAttribute`.
    ///
    /// As in VTK, setting a role-specific array replaces the previous active
    /// array for that role. This compact Rust version tracks the role by name
    /// because `FieldData` compacts array storage on removal.
    pub fn set_attribute(&mut self, array: Option<AnyArray>, attribute_type: i32) -> i32 {
        let Some(role) = DataSetAttribute::from_i32(attribute_type) else {
            return -1;
        };
        self.set_field_data_attribute(role, array.map(FieldDataArray::from_any_array))
    }

    pub(crate) fn set_field_data_attribute(
        &mut self,
        role: DataSetAttribute,
        array: Option<FieldDataArray>,
    ) -> i32 {
        let Some(array) = array else {
            if let Some(old_name) = self.active_name(role).map(str::to_string) {
                self.remove_field_data_array(&old_name);
            } else {
                self.set_active_name(role, None);
            }
            return -1;
        };

        if role != DataSetAttribute::PedigreeIds && !array.is_data_array() {
            return -1;
        }
        if !Self::has_valid_number_of_components(&array, role) {
            return -1;
        }

        let new_name = array.get_name().to_string();
        if let Some(old_name) = self.active_name(role).map(str::to_string) {
            if old_name != new_name {
                self.remove_field_data_array(&old_name);
            }
        }

        let index = self.add_field_data_array(array);
        self.set_active_name(role, Some(new_name));
        i32::try_from(index).expect("array index must fit int")
    }

    /// VTK: `vtkDataSetAttributes::SetActiveAttribute(const char*, int)`.
    pub fn set_active_attribute(&mut self, name: &str, attribute_type: i32) -> i32 {
        let Some(role) = DataSetAttribute::from_i32(attribute_type) else {
            return -1;
        };
        let index = if name.is_empty() {
            -1
        } else {
            self.storage
                .field_data
                .find_array_index(name)
                .map(|index| i32::try_from(index).expect("array index must fit int"))
                .unwrap_or(-1)
        };
        self.set_active_attribute_by_index_role(role, index)
    }

    /// VTK: `vtkDataSetAttributes::SetActiveAttribute(int, int)`.
    pub fn set_active_attribute_by_index(&mut self, index: i32, attribute_type: i32) -> i32 {
        let Some(role) = DataSetAttribute::from_i32(attribute_type) else {
            return -1;
        };
        self.set_active_attribute_by_index_role(role, index)
    }

    pub(crate) fn set_active_attribute_by_index_role(
        &mut self,
        role: DataSetAttribute,
        index: i32,
    ) -> i32 {
        if index == -1 {
            self.set_active_name(role, None);
            return -1;
        }
        if index < 0 {
            return -1;
        }

        let index = index as usize;
        let Some(array) = self.storage.field_data.get_field_data_array_by_index(index) else {
            return -1;
        };
        let name = array.get_name().to_string();

        if role != DataSetAttribute::PedigreeIds
            && (!array.is_data_array() || !Self::has_valid_number_of_components(array, role))
        {
            return -1;
        }

        self.set_active_name(role, Some(name));
        i32::try_from(index).expect("array index must fit int")
    }

    pub fn get_attribute(&self, attribute_type: i32) -> Option<&AnyArray> {
        let role = DataSetAttribute::from_i32(attribute_type)?;
        self.get_field_data_attribute(role)
            .map(FieldDataArray::get_data)
    }

    /// VTK: `vtkDataSetAttributes::GetAbstractAttribute`.
    pub fn get_abstract_attribute(&self, attribute_type: i32) -> Option<&AnyArray> {
        let role = DataSetAttribute::from_i32(attribute_type)?;
        self.get_field_data_attribute(role)
            .map(FieldDataArray::get_data)
    }

    pub(crate) fn get_field_data_attribute(
        &self,
        role: DataSetAttribute,
    ) -> Option<&FieldDataArray> {
        self.active_name(role)
            .and_then(|name| self.storage.field_data.get_field_data_array(name))
    }

    pub(crate) fn get_attribute_mut(
        &mut self,
        role: DataSetAttribute,
    ) -> Option<&mut FieldDataArray> {
        let name = self.active_name(role)?.to_string();
        self.storage_mut().field_data.get_array_mut(&name)
    }

    /// VTK: `vtkDataSetAttributes::GetAttributeIndices`.
    pub fn get_attribute_indices(&self) -> [isize; DataSetAttribute::ALL.len()] {
        let mut indices = [-1; DataSetAttribute::ALL.len()];
        for role in DataSetAttribute::ALL {
            if let Some(name) = self.active_name(role) {
                if let Some(index) = self.storage.field_data.find_array_index(name) {
                    indices[role.index()] = index as isize;
                }
            }
        }
        indices
    }

    /// VTK: `vtkDataSetAttributes::IsArrayAnAttribute`.
    pub fn is_array_an_attribute(&self, index: i32) -> i32 {
        if index < 0 {
            return -1;
        }
        let index = index as usize;
        DataSetAttribute::ALL
            .into_iter()
            .find(|&role| self.get_attribute_indices()[role.index()] == index as isize)
            .map(|role| role.index() as i32)
            .unwrap_or(-1)
    }

    pub fn set_scalars(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, SCALARS)
    }

    pub(crate) fn set_field_data_scalars(&mut self, array: Option<FieldDataArray>) -> i32 {
        self.set_field_data_attribute(DataSetAttribute::Scalars, array)
    }

    pub fn set_vectors(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, VECTORS)
    }

    #[cfg(test)]
    pub(crate) fn set_field_data_vectors(&mut self, array: Option<FieldDataArray>) -> i32 {
        self.set_field_data_attribute(DataSetAttribute::Vectors, array)
    }

    pub fn set_normals(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, NORMALS)
    }

    pub fn set_tcoords(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, TCOORDS)
    }

    pub fn set_tensors(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, TENSORS)
    }

    pub fn set_global_ids(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, GLOBALIDS)
    }

    pub fn set_pedigree_ids(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, PEDIGREEIDS)
    }

    #[cfg(test)]
    pub(crate) fn set_field_data_pedigree_ids(&mut self, array: Option<FieldDataArray>) -> i32 {
        self.set_field_data_attribute(DataSetAttribute::PedigreeIds, array)
    }

    pub fn set_tangents(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, TANGENTS)
    }

    pub fn set_rational_weights(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, RATIONALWEIGHTS)
    }

    pub fn set_higher_order_degrees(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, HIGHERORDERDEGREES)
    }

    pub fn set_process_ids(&mut self, array: Option<AnyArray>) -> i32 {
        self.set_attribute(array, PROCESSIDS)
    }

    pub fn set_active_scalars(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, SCALARS)
    }

    pub fn set_active_vectors(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, VECTORS)
    }

    pub fn set_active_normals(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, NORMALS)
    }

    pub fn set_active_tcoords(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, TCOORDS)
    }

    pub fn set_active_tensors(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, TENSORS)
    }

    pub fn set_active_global_ids(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, GLOBALIDS)
    }

    pub fn set_active_pedigree_ids(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, PEDIGREEIDS)
    }

    pub fn set_active_tangents(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, TANGENTS)
    }

    pub fn set_active_rational_weights(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, RATIONALWEIGHTS)
    }

    pub fn set_active_higher_order_degrees(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, HIGHERORDERDEGREES)
    }

    pub fn set_active_process_ids(&mut self, name: &str) -> i32 {
        self.set_active_attribute(name, PROCESSIDS)
    }

    pub fn get_scalars(&self) -> Option<&AnyArray> {
        self.get_attribute(SCALARS)
    }

    pub(crate) fn get_field_data_scalars(&self) -> Option<&FieldDataArray> {
        self.get_field_data_attribute(DataSetAttribute::Scalars)
    }

    pub(crate) fn get_scalars_mut(&mut self) -> Option<&mut FieldDataArray> {
        self.get_attribute_mut(DataSetAttribute::Scalars)
    }

    pub fn get_vectors(&self) -> Option<&AnyArray> {
        self.get_attribute(VECTORS)
    }

    #[cfg(test)]
    pub(crate) fn get_field_data_vectors(&self) -> Option<&FieldDataArray> {
        self.get_field_data_attribute(DataSetAttribute::Vectors)
    }

    pub fn get_normals(&self) -> Option<&AnyArray> {
        self.get_attribute(NORMALS)
    }

    pub fn get_tcoords(&self) -> Option<&AnyArray> {
        self.get_attribute(TCOORDS)
    }

    pub fn get_tensors(&self) -> Option<&AnyArray> {
        self.get_attribute(TENSORS)
    }

    pub fn get_global_ids(&self) -> Option<&AnyArray> {
        self.get_attribute(GLOBALIDS)
    }

    pub fn get_pedigree_ids(&self) -> Option<&AnyArray> {
        self.get_attribute(PEDIGREEIDS)
    }

    pub(crate) fn get_field_data_pedigree_ids(&self) -> Option<&FieldDataArray> {
        self.get_field_data_attribute(DataSetAttribute::PedigreeIds)
    }

    pub fn get_tangents(&self) -> Option<&AnyArray> {
        self.get_attribute(TANGENTS)
    }

    pub fn get_rational_weights(&self) -> Option<&AnyArray> {
        self.get_attribute(RATIONALWEIGHTS)
    }

    pub fn get_higher_order_degrees(&self) -> Option<&AnyArray> {
        self.get_attribute(HIGHERORDERDEGREES)
    }

    pub fn get_process_ids(&self) -> Option<&AnyArray> {
        self.get_attribute(PROCESSIDS)
    }

    #[cfg(test)]
    pub(crate) fn has_active_attributes(&self) -> bool {
        DataSetAttribute::ALL
            .into_iter()
            .any(|role| self.get_field_data_attribute(role).is_some())
    }

    /// VTK: `vtkDataSetAttributes::CopyTuple`.
    pub(crate) fn copy_tuple(
        from_data: &FieldDataArray,
        to_data: &mut FieldDataArray,
        from_id: usize,
        to_id: usize,
    ) -> Result<(), DataSetAttributesError> {
        Self::copy_tuple_values(from_data, to_data, from_id, to_id)
    }

    /// VTK: `vtkDataSetAttributes::CopyTuples(dstStart, n, srcStart)`.
    #[allow(dead_code)]
    pub(crate) fn copy_tuples(
        from_data: &FieldDataArray,
        to_data: &mut FieldDataArray,
        dst_start: usize,
        n: usize,
        src_start: usize,
    ) -> Result<(), DataSetAttributesError> {
        for offset in 0..n {
            Self::copy_tuple_values(from_data, to_data, src_start + offset, dst_start + offset)?;
        }
        Ok(())
    }

    /// VTK: `vtkDataSetAttributes::CopyTuples(vtkAbstractArray*, vtkAbstractArray*, vtkIdList*, vtkIdList*)`.
    #[allow(dead_code)]
    pub(crate) fn copy_tuples_by_ids(
        from_data: &FieldDataArray,
        to_data: &mut FieldDataArray,
        from_ids: &[VtkIdType],
        to_ids: &[VtkIdType],
    ) -> Result<(), DataSetAttributesError> {
        if from_ids.len() != to_ids.len() {
            return Err(DataSetAttributesError::TupleIdLengthMismatch {
                from_len: from_ids.len(),
                to_len: to_ids.len(),
            });
        }

        for (&from_id, &to_id) in from_ids.iter().zip(to_ids) {
            Self::copy_tuple_values(
                from_data,
                to_data,
                vtk_id_to_usize(from_id),
                vtk_id_to_usize(to_id),
            )?;
        }
        Ok(())
    }

    fn copy_tuple_values(
        from_data: &FieldDataArray,
        to_data: &mut FieldDataArray,
        from_id: usize,
        to_id: usize,
    ) -> Result<(), DataSetAttributesError> {
        let components = from_data.get_number_of_components();
        let to_components = to_data.get_number_of_components();
        if components != to_components {
            return Err(DataSetAttributesError::TupleComponentMismatch {
                from_components: components,
                to_components,
            });
        }
        if from_id >= from_data.get_number_of_tuples() {
            return Err(DataSetAttributesError::TupleOutOfRange {
                array: from_data.get_name().to_string(),
                tuple: from_id,
            });
        }

        to_data
            .copy_tuple_from(from_data, from_id, to_id)
            .then_some(())
            .ok_or(DataSetAttributesError::TupleOutOfRange {
                array: to_data.get_name().to_string(),
                tuple: to_id,
            })
    }

    fn has_valid_number_of_components(array: &FieldDataArray, role: DataSetAttribute) -> bool {
        role.component_rule()
            .accepts(array.get_number_of_components())
    }

    pub(crate) fn active_name(&self, role: DataSetAttribute) -> Option<&str> {
        match role {
            DataSetAttribute::Scalars => self.storage.active_scalars.as_deref(),
            DataSetAttribute::Vectors => self.storage.active_vectors.as_deref(),
            DataSetAttribute::Normals => self.storage.active_normals.as_deref(),
            DataSetAttribute::TCoords => self.storage.active_tcoords.as_deref(),
            DataSetAttribute::Tensors => self.storage.active_tensors.as_deref(),
            DataSetAttribute::GlobalIds => self.storage.active_global_ids.as_deref(),
            DataSetAttribute::PedigreeIds => self.storage.active_pedigree_ids.as_deref(),
            DataSetAttribute::EdgeFlag => self.storage.active_edge_flag.as_deref(),
            DataSetAttribute::Tangents => self.storage.active_tangents.as_deref(),
            DataSetAttribute::RationalWeights => self.storage.active_rational_weights.as_deref(),
            DataSetAttribute::HigherOrderDegrees => {
                self.storage.active_higher_order_degrees.as_deref()
            }
            DataSetAttribute::ProcessIds => self.storage.active_process_ids.as_deref(),
        }
    }

    fn set_active_name(&mut self, role: DataSetAttribute, name: Option<String>) {
        let storage = self.storage_mut();
        match role {
            DataSetAttribute::Scalars => storage.active_scalars = name,
            DataSetAttribute::Vectors => storage.active_vectors = name,
            DataSetAttribute::Normals => storage.active_normals = name,
            DataSetAttribute::TCoords => storage.active_tcoords = name,
            DataSetAttribute::Tensors => storage.active_tensors = name,
            DataSetAttribute::GlobalIds => storage.active_global_ids = name,
            DataSetAttribute::PedigreeIds => storage.active_pedigree_ids = name,
            DataSetAttribute::EdgeFlag => storage.active_edge_flag = name,
            DataSetAttribute::Tangents => storage.active_tangents = name,
            DataSetAttribute::RationalWeights => storage.active_rational_weights = name,
            DataSetAttribute::HigherOrderDegrees => storage.active_higher_order_degrees = name,
            DataSetAttribute::ProcessIds => storage.active_process_ids = name,
        }
    }

    fn clear_active_name(&mut self, name: &str) {
        for role in DataSetAttribute::ALL {
            if self.active_name(role) == Some(name) {
                self.set_active_name(role, None);
            }
        }
    }

    fn sync_active_after_field_data_change(&mut self) {
        for role in DataSetAttribute::ALL {
            let Some(name) = self.active_name(role).map(str::to_string) else {
                continue;
            };
            if self
                .storage
                .field_data
                .get_field_data_array(&name)
                .is_none()
            {
                self.set_active_name(role, None);
            }
        }
    }

    /// VTK: `vtkDataSetAttributes::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        let mut field_data = FieldData::new();
        field_data.deep_copy(&source.storage.field_data);
        self.storage = Arc::new(DataSetAttributesStorage {
            field_data,
            modified_time: source.storage.modified_time,
            copy_attribute_flags: source.storage.copy_attribute_flags,
            required_arrays: source.storage.required_arrays.clone(),
            target_indices: source.storage.target_indices.clone(),
            active_scalars: source.storage.active_scalars.clone(),
            active_vectors: source.storage.active_vectors.clone(),
            active_normals: source.storage.active_normals.clone(),
            active_tcoords: source.storage.active_tcoords.clone(),
            active_tensors: source.storage.active_tensors.clone(),
            active_global_ids: source.storage.active_global_ids.clone(),
            active_pedigree_ids: source.storage.active_pedigree_ids.clone(),
            active_edge_flag: source.storage.active_edge_flag.clone(),
            active_tangents: source.storage.active_tangents.clone(),
            active_rational_weights: source.storage.active_rational_weights.clone(),
            active_higher_order_degrees: source.storage.active_higher_order_degrees.clone(),
            active_process_ids: source.storage.active_process_ids.clone(),
        });
    }

    /// VTK: `vtkDataSetAttributes::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::clone(&source.storage);
    }

    pub(crate) fn deep_clone(&self) -> Self {
        let mut output = Self::new();
        output.deep_copy(self);
        output
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }

    fn storage_mut(&mut self) -> &mut DataSetAttributesStorage {
        let storage = Arc::make_mut(&mut self.storage);
        storage.modified_time = storage.modified_time.saturating_add(1);
        storage
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentRule {
    Exact(usize),
    Max(usize),
    NoLimit,
    Tensor,
}

impl ComponentRule {
    fn accepts(self, components: usize) -> bool {
        match self {
            Self::Exact(expected) => components == expected,
            Self::Max(max) => components <= max,
            Self::NoLimit => true,
            Self::Tensor => components == 9 || components == 6,
        }
    }
}
