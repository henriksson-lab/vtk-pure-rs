use crate::common::core::{AnyArray, ArrayError, Variant, VariantArray, VtkDataType, VtkIdType};
#[cfg(test)]
use crate::common::core::{DoubleArray, LongLongArray, StringArray, UnsignedCharArray};
use std::cmp::Ordering;
use std::{collections::BTreeMap, sync::Arc};

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}

fn vtk_id_from_usize(id: usize) -> VtkIdType {
    VtkIdType::try_from(id).expect("usize id must fit vtkIdType")
}

fn int_from_usize(value: usize) -> i32 {
    i32::try_from(value).expect("usize value must fit int")
}

fn write_nan_range(range: &mut [f64]) {
    assert!(range.len() >= 2, "range must hold two values");
    range[0] = f64::NAN;
    range[1] = f64::NAN;
}

impl Variant {
    #[cfg(test)]
    fn data_type(&self) -> VtkDataType {
        match self {
            Self::Invalid => VtkDataType::Void,
            Self::F64(_) => VtkDataType::Double,
            Self::I64(_) => VtkDataType::LongLong,
            Self::U8(_) => VtkDataType::UnsignedChar,
            Self::String(_) => VtkDataType::String,
        }
    }

    fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Invalid => None,
            Self::F64(value) => Some(*value),
            Self::I64(value) => Some(*value as f64),
            Self::U8(value) => Some(*value as f64),
            Self::String(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FieldDataSortKey {
    Invalid,
    Number(f64),
    String(String),
}

impl FieldDataSortKey {
    fn from_variant(value: Variant) -> Self {
        match value {
            Variant::Invalid => Self::Invalid,
            Variant::F64(value) => Self::Number(value),
            Variant::I64(value) => Self::Number(value as f64),
            Variant::U8(value) => Self::Number(value as f64),
            Variant::String(value) => Self::String(value),
        }
    }
}

impl PartialOrd for FieldDataSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Invalid, Self::Invalid) => Some(Ordering::Equal),
            (Self::Invalid, _) => Some(Ordering::Less),
            (_, Self::Invalid) => Some(Ordering::Greater),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
            (Self::Number(_), Self::String(_)) => Some(Ordering::Less),
            (Self::String(_), Self::Number(_)) => Some(Ordering::Greater),
        }
    }
}

/// Named array held by `FieldData`.
///
/// VTK origin: `VTK/Common/DataModel/vtkFieldData.cxx`, where field data stores
/// `vtkAbstractArray*` entries. This Rust type keeps only the basics needed by
/// the new crate until richer typed arrays exist locally.
#[derive(Debug)]
pub(crate) struct FieldDataArray {
    modified_time: u64,
    array: AnyArray,
}

impl Clone for FieldDataArray {
    fn clone(&self) -> Self {
        self.shallow_clone()
    }
}

impl PartialEq for FieldDataArray {
    fn eq(&self, other: &Self) -> bool {
        self.array == other.array
    }
}

impl FieldDataArray {
    pub(crate) fn from_any_array(array: AnyArray) -> Self {
        Self {
            modified_time: 0,
            array,
        }
    }

    pub(crate) fn new_with_data_type(
        name: impl Into<String>,
        number_of_components: usize,
        data_type: VtkDataType,
    ) -> Self {
        let mut array = AnyArray::create_array(data_type)
            .unwrap_or_else(|| AnyArray::Variant(VariantArray::new()));
        array.set_name(name);
        array.set_number_of_components(number_of_components.max(1) as i32);
        Self {
            modified_time: 0,
            array,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_values(
        name: impl Into<String>,
        number_of_components: usize,
        values: Vec<Variant>,
    ) -> Self {
        let data_type = values
            .first()
            .map_or(VtkDataType::Variant, Variant::data_type);
        Self::from_values_with_data_type(name, number_of_components, data_type, values)
    }

    #[cfg(test)]
    pub(crate) fn from_values_with_data_type(
        name: impl Into<String>,
        number_of_components: usize,
        data_type: VtkDataType,
        values: Vec<Variant>,
    ) -> Self {
        let number_of_components = number_of_components.max(1);
        assert!(
            values.len() % number_of_components == 0,
            "field-data values must contain a whole number of tuples"
        );
        let mut output = Self::new_with_data_type(name, number_of_components, data_type);
        output.replace_values(values);
        output.modified_time = 0;
        output
    }

    #[cfg(test)]
    pub(crate) fn from_f64(
        name: impl Into<String>,
        number_of_components: usize,
        values: Vec<f64>,
    ) -> Self {
        Self::from_values_with_data_type(
            name,
            number_of_components,
            VtkDataType::Double,
            values.into_iter().map(Variant::F64).collect(),
        )
    }

    #[cfg(test)]
    pub(crate) fn from_i64(
        name: impl Into<String>,
        number_of_components: usize,
        values: Vec<i64>,
    ) -> Self {
        Self::from_values_with_data_type(
            name,
            number_of_components,
            VtkDataType::LongLong,
            values.into_iter().map(Variant::I64).collect(),
        )
    }

    #[cfg(test)]
    pub(crate) fn from_u8(
        name: impl Into<String>,
        number_of_components: usize,
        values: Vec<u8>,
    ) -> Self {
        Self::from_values_with_data_type(
            name,
            number_of_components,
            VtkDataType::UnsignedChar,
            values.into_iter().map(Variant::U8).collect(),
        )
    }

    #[cfg(test)]
    pub(crate) fn from_strings(name: impl Into<String>, values: Vec<String>) -> Self {
        Self::from_values_with_data_type(
            name,
            1,
            VtkDataType::String,
            values.into_iter().map(Variant::String).collect(),
        )
    }

    /// VTK: `vtkAbstractArray::GetName`.
    pub fn get_name(&self) -> &str {
        self.array.get_name()
    }

    /// VTK: `vtkAbstractArray::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> usize {
        self.array.get_number_of_components() as usize
    }

    /// VTK: `vtkAbstractArray::GetDataType`.
    pub fn get_data_type(&self) -> VtkDataType {
        self.array.get_data_type()
    }

    /// VTK: `vtkAbstractArray::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> usize {
        self.array.get_number_of_tuples() as usize
    }

    pub(crate) fn is_data_array(&self) -> bool {
        self.array.is_data_array()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: usize) {
        let len = number_of_tuples
            .checked_mul(self.get_number_of_components())
            .expect("field-data tuple resize overflow");
        self.array.set_number_of_tuples(number_of_tuples as i64);
        debug_assert_eq!(self.array.get_number_of_values(), len as i64);
        self.modified();
    }

    fn copy_tuple_within(&mut self, from_tuple: usize, to_tuple: usize) -> bool {
        let components = self.get_number_of_components();
        if from_tuple >= self.get_number_of_tuples() {
            return false;
        }

        let tuple = self.tuple_values(from_tuple);
        if to_tuple >= self.get_number_of_tuples() {
            self.set_number_of_tuples(to_tuple + 1);
        }
        debug_assert_eq!(tuple.len(), components);
        self.set_tuple_values(to_tuple, &tuple)
    }

    pub(crate) fn copy_tuple_from(
        &mut self,
        source: &Self,
        from_tuple: usize,
        to_tuple: usize,
    ) -> bool {
        if source.get_number_of_components() != self.get_number_of_components()
            || from_tuple >= source.get_number_of_tuples()
        {
            return false;
        }
        self.array
            .copy_tuple_from(&source.array, from_tuple, to_tuple)
            .is_ok()
    }

    /// VTK: `vtkAbstractArray::InterpolateTuple(vtkIdList*, weights)`.
    pub fn interpolate_tuple_from(
        &mut self,
        source: &Self,
        source_tuples: &[usize],
        weights: &[f64],
        to_tuple: usize,
    ) -> bool {
        if source.get_number_of_components() != self.get_number_of_components()
            || source_tuples.is_empty()
            || source_tuples.len() != weights.len()
            || source_tuples
                .iter()
                .any(|&tuple| tuple >= source.get_number_of_tuples())
        {
            return false;
        }

        self.array
            .interpolate_tuple_from(&source.array, source_tuples, weights, to_tuple)
    }

    pub(crate) fn remove_tuple_swap_with_last(&mut self, tuple: usize) -> bool {
        let number_of_tuples = self.get_number_of_tuples();
        if tuple >= number_of_tuples {
            return false;
        }

        let last_tuple = number_of_tuples - 1;
        if tuple != last_tuple {
            self.copy_tuple_within(last_tuple, tuple);
        }
        self.set_number_of_tuples(last_tuple);
        true
    }

    pub(crate) fn values_as_variants(&self) -> Vec<Variant> {
        (0..self.array.get_number_of_values())
            .map(|value_idx| self.get_value(value_idx as usize))
            .collect()
    }

    pub fn get_value(&self, value_idx: usize) -> Variant {
        match &self.array {
            AnyArray::String(array) => Variant::String(array.as_slice()[value_idx].clone()),
            AnyArray::Variant(array) => array.as_slice()[value_idx].clone(),
            array if array.is_numeric() => {
                let tuple = value_idx / self.get_number_of_components();
                let component = value_idx % self.get_number_of_components();
                let value = array
                    .numeric_tuple_as_f64_checked(tuple)
                    .expect("numeric field-data array")[component];
                variant_from_numeric(value, self.get_data_type())
            }
            _ => Variant::F64(0.0),
        }
    }

    #[cfg(test)]
    pub(crate) fn set_value(&mut self, value_idx: usize, value: Variant) -> bool {
        let tuple = value_idx / self.get_number_of_components();
        let component = value_idx % self.get_number_of_components();
        if tuple >= self.get_number_of_tuples() {
            self.set_number_of_tuples(tuple + 1);
        }
        let mut values = self.tuple_values(tuple);
        values[component] = value;
        self.set_tuple_values(tuple, &values)
    }

    pub fn get_data(&self) -> &AnyArray {
        &self.array
    }

    pub(crate) fn get_data_mut(&mut self) -> &mut AnyArray {
        self.modified();
        &mut self.array
    }

    pub(crate) fn insert_numeric_tuple_from_f64(
        &mut self,
        tuple_idx: usize,
        tuple: &[f64],
    ) -> bool {
        if self
            .array
            .insert_numeric_tuple_from_f64_checked(tuple_idx, tuple)
            .is_ok()
        {
            self.modified();
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.array.capacity()
    }

    pub fn squeeze(&mut self) {
        self.array.squeeze();
        self.modified();
    }

    pub fn reset(&mut self) {
        self.array.reset();
        self.modified();
    }

    pub fn get_actual_memory_size(&self) -> usize {
        self.array.get_actual_memory_size()
    }

    /// VTK: `vtkAbstractArray::ReserveValues`.
    pub fn reserve_values(&mut self, values: usize) -> bool {
        let status = self.array.reserve_values(values as i64);
        self.modified();
        status
    }

    /// VTK: `vtkAbstractArray::Initialize`.
    pub fn initialize(&mut self) {
        self.array.initialize();
        self.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.modified_time
    }

    /// VTK: `vtkAbstractArray::DeepCopy`.
    pub(crate) fn deep_clone(&self) -> Self {
        Self {
            modified_time: self.modified_time,
            array: self.array.deep_clone(),
        }
    }

    /// VTK: `vtkAbstractArray` reference-counted copy.
    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            modified_time: self.modified_time,
            array: self.array.shallow_clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_values_with(&self, other: &Self) -> bool {
        self.array.shares_storage_with(&other.array)
    }

    #[cfg(test)]
    fn replace_values(&mut self, values: Vec<Variant>) {
        let number_of_components = self.get_number_of_components();
        self.array = any_array_from_variants(
            self.get_name(),
            number_of_components,
            self.get_data_type(),
            values,
        );
    }

    fn tuple_values(&self, tuple: usize) -> Vec<Variant> {
        let start = tuple * self.get_number_of_components();
        (start..start + self.get_number_of_components())
            .map(|value_idx| self.get_value(value_idx))
            .collect()
    }

    fn component_sort_key(&self, tuple: usize, component: usize) -> Option<FieldDataSortKey> {
        if tuple >= self.get_number_of_tuples() || component >= self.get_number_of_components() {
            return None;
        }
        Some(FieldDataSortKey::from_variant(self.get_value(
            tuple * self.get_number_of_components() + component,
        )))
    }

    fn set_tuple_values(&mut self, tuple: usize, values: &[Variant]) -> bool {
        if values.len() != self.get_number_of_components() {
            return false;
        }
        if tuple >= self.get_number_of_tuples() {
            self.set_number_of_tuples(tuple + 1);
        }
        let data_type = self.get_data_type();
        let result = match &mut self.array {
            AnyArray::String(array) => {
                let strings: Vec<_> = values
                    .iter()
                    .map(|value| match value {
                        Variant::String(value) => value.clone(),
                        Variant::Invalid | Variant::F64(_) | Variant::I64(_) | Variant::U8(_) => {
                            String::new()
                        }
                    })
                    .collect();
                array.insert_typed_tuple(tuple as VtkIdType, &strings);
                Ok(())
            }
            AnyArray::Variant(array) => {
                array.insert_typed_tuple(tuple as VtkIdType, values);
                Ok(())
            }
            array if array.is_numeric() => {
                let numbers: Option<Vec<_>> = values.iter().map(Variant::to_f64).collect();
                numbers
                    .ok_or(ArrayError::TypeMismatch {
                        destination: data_type,
                        source_type: VtkDataType::String,
                    })
                    .and_then(|numbers| {
                        array.insert_numeric_tuple_from_f64_checked(tuple, &numbers)
                    })
            }
            _ => Err(ArrayError::UnsupportedDataType(data_type)),
        };
        if result.is_ok() {
            self.modified();
            true
        } else {
            false
        }
    }

    fn shuffle_tuples_by_indices(&mut self, idx: &[VtkIdType], dir: i32) -> bool {
        if self.get_number_of_tuples() != idx.len() {
            return false;
        }

        let input: Vec<_> = (0..idx.len())
            .map(|tuple_idx| self.tuple_values(tuple_idx))
            .collect();
        for output_tuple in 0..idx.len() {
            let index_idx = if dir == 0 {
                output_tuple
            } else {
                idx.len() - 1 - output_tuple
            };
            let Ok(input_tuple) = usize::try_from(idx[index_idx]) else {
                return false;
            };
            let Some(tuple_values) = input.get(input_tuple) else {
                return false;
            };
            if !self.set_tuple_values(output_tuple, tuple_values) {
                return false;
            }
        }
        true
    }

    fn modified(&mut self) {
        self.modified_time = self.modified_time.saturating_add(1);
    }
}

#[cfg(test)]
fn any_array_from_variants(
    name: &str,
    number_of_components: usize,
    data_type: VtkDataType,
    values: Vec<Variant>,
) -> AnyArray {
    match data_type {
        VtkDataType::String => AnyArray::String(StringArray::from_vec(
            name,
            values
                .into_iter()
                .map(|value| match value {
                    Variant::String(value) => value,
                    _ => String::new(),
                })
                .collect(),
            number_of_components,
        )),
        VtkDataType::UnsignedChar => AnyArray::UnsignedChar(UnsignedCharArray::from_vec(
            name,
            values
                .into_iter()
                .map(|value| match value {
                    Variant::U8(value) => value,
                    Variant::I64(value) => value as u8,
                    Variant::F64(value) => value as u8,
                    Variant::Invalid | Variant::String(_) => 0,
                })
                .collect(),
            number_of_components,
        )),
        VtkDataType::LongLong | VtkDataType::IdType => AnyArray::LongLong(LongLongArray::from_vec(
            name,
            values
                .into_iter()
                .map(|value| match value {
                    Variant::I64(value) => value,
                    Variant::U8(value) => value as i64,
                    Variant::F64(value) => value as i64,
                    Variant::Invalid | Variant::String(_) => 0,
                })
                .collect(),
            number_of_components,
        )),
        VtkDataType::Variant => AnyArray::Variant(VariantArray::from_values(
            name,
            values,
            number_of_components,
        )),
        _ => AnyArray::Double(DoubleArray::from_vec(
            name,
            values
                .into_iter()
                .map(|value| value.to_f64().unwrap_or_default())
                .collect(),
            number_of_components,
        )),
    }
}

fn variant_from_numeric(value: f64, data_type: VtkDataType) -> Variant {
    match data_type {
        VtkDataType::UnsignedChar
        | VtkDataType::UnsignedShort
        | VtkDataType::UnsignedInt
        | VtkDataType::UnsignedLong
        | VtkDataType::UnsignedLongLong
        | VtkDataType::Bit
        | VtkDataType::Char => Variant::U8(value as u8),
        VtkDataType::LongLong
        | VtkDataType::IdType
        | VtkDataType::Long
        | VtkDataType::Int
        | VtkDataType::Short
        | VtkDataType::SignedChar => Variant::I64(value as i64),
        _ => Variant::F64(value),
    }
}

/// Shared storage for `FieldData` collection state.
#[derive(Debug, Clone, PartialEq)]
struct FieldDataStorage {
    arrays: Vec<FieldDataArray>,
    reserved_array_slots: usize,
    modified_time: u64,
    do_copy_all_on: bool,
    do_copy_all_off: bool,
    ghosts_to_skip: u8,
    copy_field_flags: BTreeMap<String, bool>,
}

/// A named collection of field arrays.
///
/// VTK origin: `VTK/Common/DataModel/vtkFieldData.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldData {
    storage: Arc<FieldDataStorage>,
}

impl FieldData {
    /// VTK: `vtkFieldData::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(FieldDataStorage {
                arrays: Vec::new(),
                reserved_array_slots: 0,
                modified_time: 0,
                do_copy_all_on: true,
                do_copy_all_off: false,
                ghosts_to_skip: 0,
                copy_field_flags: BTreeMap::new(),
            }),
        }
    }

    /// VTK: `vtkFieldData::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    /// VTK: `vtkFieldData::AddArray`.
    pub fn add_array(&mut self, array: AnyArray) -> i32 {
        int_from_usize(self.add_field_data_array(FieldDataArray::from_any_array(array)))
    }

    pub(crate) fn add_field_data_array(&mut self, array: FieldDataArray) -> usize {
        let index = self
            .storage
            .arrays
            .iter()
            .position(|existing| existing.get_name() == array.get_name())
            .unwrap_or(self.storage.arrays.len());
        self.set_array(int_from_usize(index), array)
            .expect("AddArray must choose a valid active or append index")
    }

    /// VTK: `vtkFieldData::GetArray(const char*)`.
    pub fn get_array(&self, name: &str) -> Option<&AnyArray> {
        let mut index = -1;
        self.get_array_with_index(name, &mut index)
    }

    /// VTK: `vtkFieldData::GetArray(const char*, int&)`.
    pub fn get_array_with_index(&self, name: &str, index: &mut i32) -> Option<&AnyArray> {
        let Some((array_index, array)) = self.find_field_data_array(name) else {
            *index = -1;
            return None;
        };
        if array.is_data_array() {
            *index = int_from_usize(array_index);
            Some(array.get_data())
        } else {
            *index = -1;
            None
        }
    }

    /// VTK: `vtkFieldData::GetArray(int)`.
    pub fn get_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.get_field_data_array_by_index_i32(index)
            .filter(|array| array.is_data_array())
            .map(FieldDataArray::get_data)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*)`.
    pub fn get_abstract_array(&self, name: &str) -> Option<&AnyArray> {
        let mut index = -1;
        self.get_abstract_array_with_index(name, &mut index)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*, int&)`.
    pub fn get_abstract_array_with_index(&self, name: &str, index: &mut i32) -> Option<&AnyArray> {
        let Some((array_index, array)) = self.find_field_data_array(name) else {
            *index = -1;
            return None;
        };
        *index = int_from_usize(array_index);
        Some(array.get_data())
    }

    /// VTK: `vtkFieldData::GetAbstractArray(int)`.
    pub fn get_abstract_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.get_field_data_array_by_index_i32(index)
            .map(FieldDataArray::get_data)
    }

    pub(crate) fn get_field_data_array(&self, name: &str) -> Option<&FieldDataArray> {
        self.storage
            .arrays
            .iter()
            .find(|array| array.get_name() == name)
    }

    fn find_field_data_array(&self, name: &str) -> Option<(usize, &FieldDataArray)> {
        self.storage
            .arrays
            .iter()
            .enumerate()
            .find(|(_, array)| array.get_name() == name)
    }

    pub(crate) fn get_array_mut(&mut self, name: &str) -> Option<&mut FieldDataArray> {
        self.storage_mut()
            .arrays
            .iter_mut()
            .find(|array| array.get_name() == name)
    }

    pub(crate) fn get_field_data_array_by_index(&self, index: usize) -> Option<&FieldDataArray> {
        self.storage.arrays.get(index)
    }

    fn get_field_data_array_by_index_i32(&self, index: i32) -> Option<&FieldDataArray> {
        let index = usize::try_from(index).ok()?;
        self.get_field_data_array_by_index(index)
    }

    pub(crate) fn arrays(&self) -> &[FieldDataArray] {
        &self.storage.arrays
    }

    pub(crate) fn arrays_mut(&mut self) -> &mut [FieldDataArray] {
        &mut self.storage_mut().arrays
    }

    pub(crate) fn sort_tuples_by_component(
        &mut self,
        array_name: &str,
        component: i32,
    ) -> Option<Vec<VtkIdType>> {
        let Some((_, array)) = self.find_field_data_array(array_name) else {
            return None;
        };
        let component = usize::try_from(component).ok()?;
        if component >= array.get_number_of_components() {
            return None;
        }

        let number_of_tuples = array.get_number_of_tuples();
        if number_of_tuples == 0 {
            return None;
        }

        let mut idx: Vec<VtkIdType> = (0..number_of_tuples).map(vtk_id_from_usize).collect();
        idx.sort_by(|left, right| {
            let left_tuple = vtk_id_to_usize(*left);
            let right_tuple = vtk_id_to_usize(*right);
            let left_key = array.component_sort_key(left_tuple, component);
            let right_key = array.component_sort_key(right_tuple, component);
            left_key.partial_cmp(&right_key).unwrap_or(Ordering::Equal)
        });
        Some(idx)
    }

    pub(crate) fn shuffle_arrays_with_tuple_count(
        &mut self,
        number_of_tuples: usize,
        idx: &[VtkIdType],
        dir: i32,
    ) {
        for array in self
            .arrays_mut()
            .iter_mut()
            .filter(|array| array.get_number_of_tuples() == number_of_tuples)
        {
            array.shuffle_tuples_by_indices(idx, dir);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FieldDataArray> {
        self.storage.arrays.iter()
    }

    fn compute_array_range(
        &self,
        array: &AnyArray,
        range: &mut [f64],
        component: i32,
        finite_only: bool,
    ) -> bool {
        if !array.is_data_array()
            || !(component < array.get_number_of_components() || component == -1)
        {
            write_nan_range(range);
            return false;
        }
        if finite_only {
            array.compute_finite_range(range, component)
        } else {
            array.compute_range(range, component)
        }
    }

    /// VTK: `vtkFieldData::GetNumberOfArrays`.
    pub fn get_number_of_arrays(&self) -> i32 {
        int_from_usize(self.storage.arrays.len())
    }

    /// VTK: `vtkFieldData::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        int_from_usize(
            self.storage
                .arrays
                .iter()
                .map(FieldDataArray::get_number_of_components)
                .sum(),
        )
    }

    /// VTK: `vtkFieldData::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        vtk_id_from_usize(
            self.storage
                .arrays
                .first()
                .map_or(0, FieldDataArray::get_number_of_tuples),
        )
    }

    /// VTK: `vtkFieldData::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = format!("Number Of Arrays: {}\n", self.get_number_of_arrays());
        for index in 0..self.storage.arrays.len() {
            let name = self
                .get_array_name(int_from_usize(index))
                .filter(|name| !name.is_empty())
                .unwrap_or("nullptr");
            output.push_str(&format!("Array {} name = {}\n", index, name));
        }
        output.push_str(&format!(
            "Number Of Components: {}\nNumber Of Tuples: {}\n",
            self.get_number_of_components(),
            self.get_number_of_tuples()
        ));
        output
    }

    /// VTK: `vtkFieldData::HasArray`.
    pub fn has_array(&self, name: &str) -> bool {
        self.get_abstract_array(name).is_some()
    }

    /// VTK: `vtkFieldData::GetArrayName`.
    pub fn get_array_name(&self, index: i32) -> Option<&str> {
        self.get_field_data_array_by_index_i32(index)
            .map(FieldDataArray::get_name)
    }

    /// VTK: `vtkFieldData::GhostArrayName`.
    pub fn ghost_array_name() -> &'static str {
        "vtkGhostType"
    }

    /// VTK: `vtkFieldData::GetGhostsToSkip`.
    pub fn get_ghosts_to_skip(&self) -> u8 {
        self.storage.ghosts_to_skip
    }

    /// VTK: `vtkFieldData::SetGhostsToSkip`.
    pub fn set_ghosts_to_skip(&mut self, ghosts_to_skip: u8) {
        if self.storage.ghosts_to_skip != ghosts_to_skip {
            self.storage_mut().ghosts_to_skip = ghosts_to_skip;
        }
    }

    /// VTK: `vtkFieldData::GetGhostArray`.
    pub fn get_ghost_array(&self) -> Option<&AnyArray> {
        let array = self.get_abstract_array(Self::ghost_array_name())?;
        array.as_unsigned_char_array().map(|_| array)
    }

    /// VTK: `vtkFieldData::HasAnyGhostBitSet`.
    pub fn has_any_ghost_bit_set(&self, bit_flag: i32) -> bool {
        let Some(AnyArray::UnsignedChar(array)) = self.get_ghost_array() else {
            return false;
        };
        let bit_flag = bit_flag as u8;
        array.as_slice().iter().any(|value| value & bit_flag != 0)
    }

    /// VTK: `vtkFieldData::NullData`.
    pub fn null_data(&mut self, id: VtkIdType) {
        let Ok(tuple_idx) = usize::try_from(id) else {
            return;
        };
        for array in self
            .arrays_mut()
            .iter_mut()
            .filter(|array| array.is_data_array())
        {
            let tuple = vec![0.0; array.get_number_of_components()];
            array.insert_numeric_tuple_from_f64(tuple_idx, &tuple);
        }
    }

    /// VTK: `vtkFieldData::GetRange(const char*, double[2], int)`.
    pub fn get_range(&self, name: &str, range: &mut [f64], component: i32) -> bool {
        let Some(index) = self.find_array_index(name) else {
            write_nan_range(range);
            return false;
        };
        self.get_range_by_index(int_from_usize(index), range, component)
    }

    /// VTK: `vtkFieldData::GetRange(int, double[2], int)`.
    pub fn get_range_by_index(&self, index: i32, range: &mut [f64], component: i32) -> bool {
        let Some(array) = self.get_abstract_array_by_index(index) else {
            write_nan_range(range);
            return false;
        };
        self.compute_array_range(array, range, component, false)
    }

    /// VTK: `vtkFieldData::GetFiniteRange(const char*, double[2], int)`.
    pub fn get_finite_range(&self, name: &str, range: &mut [f64], component: i32) -> bool {
        let Some(index) = self.find_array_index(name) else {
            write_nan_range(range);
            return false;
        };
        self.get_finite_range_by_index(int_from_usize(index), range, component)
    }

    /// VTK: `vtkFieldData::GetFiniteRange(int, double[2], int)`.
    pub fn get_finite_range_by_index(&self, index: i32, range: &mut [f64], component: i32) -> bool {
        let Some(array) = self.get_abstract_array_by_index(index) else {
            write_nan_range(range);
            return false;
        };
        self.compute_array_range(array, range, component, true)
    }

    #[cfg(test)]
    pub(crate) fn names(&self) -> Vec<&str> {
        self.storage
            .arrays
            .iter()
            .map(FieldDataArray::get_name)
            .collect()
    }

    /// VTK: `vtkFieldData::RemoveArray(const char*)`.
    pub fn remove_array(&mut self, name: &str) {
        self.remove_field_data_array(name);
    }

    /// VTK: `vtkFieldData::RemoveArray(int)`.
    pub fn remove_array_by_index(&mut self, index: i32) {
        self.remove_field_data_array_by_index_i32(index);
    }

    pub(crate) fn remove_field_data_array(&mut self, name: &str) -> Option<FieldDataArray> {
        self.find_array_index(name).map(|index| {
            self.remove_field_data_array_by_index(index)
                .expect("valid array index")
        })
    }

    pub(crate) fn remove_field_data_array_by_index(
        &mut self,
        index: usize,
    ) -> Option<FieldDataArray> {
        let storage = self.storage_mut();
        (index < storage.arrays.len()).then(|| storage.arrays.remove(index))
    }

    fn remove_field_data_array_by_index_i32(&mut self, index: i32) -> Option<FieldDataArray> {
        let index = usize::try_from(index).ok()?;
        self.remove_field_data_array_by_index(index)
    }

    /// VTK: protected `vtkFieldData::SetArray`.
    pub(crate) fn set_array(&mut self, index: i32, array: FieldDataArray) -> Option<usize> {
        let index = usize::try_from(index).ok()?;
        if index > self.storage.arrays.len() {
            return None;
        }

        if index >= self.storage.reserved_array_slots {
            self.allocate_arrays(int_from_usize(index + 1));
        }

        let storage = self.storage_mut();
        if index == storage.arrays.len() {
            storage.arrays.push(array);
        } else {
            storage.arrays[index] = array;
        }
        storage.reserved_array_slots = storage.reserved_array_slots.max(storage.arrays.len());
        Some(index)
    }

    /// VTK: `vtkFieldData::Initialize`.
    pub fn initialize(&mut self) {
        self.initialize_fields();
        self.copy_all_on();
        self.storage_mut().copy_field_flags.clear();
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.storage.arrays.is_empty()
    }

    /// VTK: `vtkFieldData::Allocate`.
    pub fn allocate(&mut self, values_per_array: VtkIdType, _ext: VtkIdType) -> bool {
        let values_per_array = vtk_id_to_usize(values_per_array);
        let mut status = false;
        for array in &mut self.storage_mut().arrays {
            array.initialize();
            status = array.reserve_values(values_per_array);
            if !status {
                break;
            }
        }
        status
    }

    /// VTK: `vtkFieldData::AllocateArrays`.
    ///
    /// VTK separates pointer slots from active arrays. This Vec-backed version
    /// keeps active arrays compact and treats extra slots as reservation.
    pub fn allocate_arrays(&mut self, number_of_arrays: i32) {
        let number_of_arrays =
            usize::try_from(number_of_arrays.max(0)).expect("non-negative VTK int must fit usize");
        if number_of_arrays == 0 {
            self.initialize();
            return;
        }

        let storage = self.storage_mut();
        if number_of_arrays < storage.arrays.len() {
            storage.arrays.truncate(number_of_arrays);
        } else {
            storage
                .arrays
                .reserve(number_of_arrays.saturating_sub(storage.arrays.len()));
        }
        storage.reserved_array_slots = number_of_arrays;
    }

    #[cfg(test)]
    fn reserved_array_slots(&self) -> usize {
        self.storage
            .reserved_array_slots
            .max(self.storage.arrays.len())
    }

    /// VTK: `vtkFieldData::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage
            .arrays
            .iter()
            .map(FieldDataArray::get_actual_memory_size)
            .sum()
    }

    /// VTK: `vtkFieldData::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .arrays
            .iter()
            .map(FieldDataArray::get_m_time)
            .fold(self.storage.modified_time, u64::max)
    }

    /// VTK: `vtkFieldData::CopyAllOn`.
    pub fn copy_all_on(&mut self) {
        let storage = self.storage_mut();
        storage.do_copy_all_on = true;
        storage.do_copy_all_off = false;
    }

    /// VTK: `vtkFieldData::CopyAllOff`.
    pub fn copy_all_off(&mut self) {
        let storage = self.storage_mut();
        storage.do_copy_all_on = false;
        storage.do_copy_all_off = true;
    }

    /// VTK: `vtkFieldData::CopyFieldOnOff`.
    fn copy_field_on_off(&mut self, field: impl Into<String>, copy: bool) {
        self.storage_mut()
            .copy_field_flags
            .insert(field.into(), copy);
    }

    /// VTK: `vtkFieldData::CopyFieldOn`.
    pub fn copy_field_on(&mut self, field: impl Into<String>) {
        self.copy_field_on_off(field, true);
    }

    /// VTK: `vtkFieldData::CopyFieldOff`.
    pub fn copy_field_off(&mut self, field: impl Into<String>) {
        self.copy_field_on_off(field, false);
    }

    /// VTK: `vtkFieldData::GetFlag`.
    #[cfg(test)]
    pub(crate) fn get_flag(&self, field: &str) -> Option<bool> {
        self.storage.copy_field_flags.get(field).copied()
    }

    /// VTK: `vtkFieldData::GetFlag` plus CopyAll fallback.
    pub(crate) fn should_copy_array(&self, name: &str) -> bool {
        self.should_copy(name)
    }

    /// VTK: `vtkFieldData::CopyFlags`.
    #[cfg(test)]
    pub(crate) fn copy_flags(&mut self, source: &Self) {
        let storage = self.storage_mut();
        storage.do_copy_all_on = source.storage.do_copy_all_on;
        storage.do_copy_all_off = source.storage.do_copy_all_off;
        storage
            .copy_field_flags
            .clone_from(&source.storage.copy_field_flags);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkFieldData::PassData`.
    ///
    /// Passed arrays are shallow copies, matching VTK's reference-counted
    /// transfer of array pointers.
    pub fn pass_data(&mut self, source: &Self) {
        let arrays_to_pass: Vec<_> = source
            .storage
            .arrays
            .iter()
            .filter(|array| self.should_copy(array.get_name()))
            .map(FieldDataArray::shallow_clone)
            .collect();

        self.allocate_arrays(int_from_usize(
            self.storage.arrays.len() + arrays_to_pass.len(),
        ));
        for array in arrays_to_pass {
            self.add_field_data_array(array);
        }
    }

    /// VTK: `vtkFieldData::Squeeze`.
    pub fn squeeze(&mut self) {
        for array in self.storage_mut().arrays.iter_mut() {
            array.squeeze();
        }
    }

    /// VTK: `vtkFieldData::Reset`.
    pub fn reset(&mut self) {
        for array in self.storage_mut().arrays.iter_mut() {
            array.reset();
        }
    }

    /// VTK: `vtkFieldData::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let number_of_tuples = vtk_id_to_usize(number_of_tuples);
        for array in self.storage_mut().arrays.iter_mut() {
            array.set_number_of_tuples(number_of_tuples);
        }
    }

    /// VTK: `vtkFieldData::SetTuple`.
    pub fn set_tuple(&mut self, dst_tuple: VtkIdType, src_tuple: VtkIdType, source: &Self) {
        assert_eq!(
            self.get_number_of_arrays(),
            source.get_number_of_arrays(),
            "field array count mismatch"
        );
        let dst_tuple = vtk_id_to_usize(dst_tuple);
        let src_tuple = vtk_id_to_usize(src_tuple);
        self.storage_mut()
            .arrays
            .iter_mut()
            .zip(source.storage.arrays.iter())
            .for_each(|(dst, src)| {
                assert!(
                    dst_tuple < dst.get_number_of_tuples(),
                    "destination tuple index out of range"
                );
                assert!(
                    dst.copy_tuple_from(src, src_tuple, dst_tuple),
                    "field tuple copy failed"
                );
            });
    }

    /// VTK: `vtkFieldData::InsertTuple`.
    pub fn insert_tuple(&mut self, dst_tuple: VtkIdType, src_tuple: VtkIdType, source: &Self) {
        assert_eq!(
            self.get_number_of_arrays(),
            source.get_number_of_arrays(),
            "field array count mismatch"
        );
        let dst_tuple = vtk_id_to_usize(dst_tuple);
        let src_tuple = vtk_id_to_usize(src_tuple);
        self.storage_mut()
            .arrays
            .iter_mut()
            .zip(source.storage.arrays.iter())
            .for_each(|(dst, src)| {
                assert!(
                    dst.copy_tuple_from(src, src_tuple, dst_tuple),
                    "field tuple copy failed"
                );
            });
    }

    /// VTK: `vtkFieldData::InsertNextTuple`.
    pub fn insert_next_tuple(&mut self, src_tuple: VtkIdType, source: &Self) -> VtkIdType {
        let dst_tuple = self.get_number_of_tuples();
        self.insert_tuple(dst_tuple, src_tuple, source);
        dst_tuple
    }

    /// VTK: `vtkFieldData::GetField`.
    pub fn get_field(&self, tuple_ids: &[VtkIdType], output: &mut Self) {
        for (dst_tuple, &src_tuple) in tuple_ids.iter().enumerate() {
            output.insert_tuple(vtk_id_from_usize(dst_tuple), src_tuple, self);
        }
    }

    /// VTK: `vtkFieldData::GetArrayContainingComponent`.
    pub fn get_array_containing_component(&self, component: i32, array_comp: &mut i32) -> i32 {
        let mut offset = 0;
        for (array_idx, array) in self.storage.arrays.iter().enumerate() {
            let next = offset + int_from_usize(array.get_number_of_components());
            if component < next {
                *array_comp = component - offset;
                return int_from_usize(array_idx);
            }
            offset = next;
        }
        -1
    }

    /// VTK: `vtkFieldData::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        let arrays: Vec<_> = source
            .storage
            .arrays
            .iter()
            .map(FieldDataArray::deep_clone)
            .collect();
        self.storage = Arc::new(FieldDataStorage {
            reserved_array_slots: source.storage.reserved_array_slots.max(arrays.len()),
            arrays,
            modified_time: source.storage.modified_time,
            do_copy_all_on: source.storage.do_copy_all_on,
            do_copy_all_off: source.storage.do_copy_all_off,
            ghosts_to_skip: source.storage.ghosts_to_skip,
            copy_field_flags: source.storage.copy_field_flags.clone(),
        });
    }

    /// VTK: `vtkFieldData::CopyStructure`.
    pub fn copy_structure(&mut self, source: &Self) {
        let arrays: Vec<_> = source
            .storage
            .arrays
            .iter()
            .map(|array| {
                FieldDataArray::new_with_data_type(
                    array.get_name(),
                    array.get_number_of_components(),
                    array.get_data_type(),
                )
            })
            .collect();
        self.storage = Arc::new(FieldDataStorage {
            reserved_array_slots: source.storage.reserved_array_slots.max(arrays.len()),
            arrays,
            modified_time: self.storage.modified_time.saturating_add(1),
            do_copy_all_on: self.storage.do_copy_all_on,
            do_copy_all_off: self.storage.do_copy_all_off,
            ghosts_to_skip: source.storage.ghosts_to_skip,
            copy_field_flags: self.storage.copy_field_flags.clone(),
        });
    }

    /// VTK: `vtkFieldData::ShallowCopy`.
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

    fn storage_mut(&mut self) -> &mut FieldDataStorage {
        let storage = Arc::make_mut(&mut self.storage);
        storage.modified_time = storage.modified_time.saturating_add(1);
        storage
    }

    fn initialize_fields(&mut self) {
        let storage = self.storage_mut();
        storage.arrays.clear();
        storage.reserved_array_slots = 0;
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*, int&)`.
    pub(crate) fn find_array_index(&self, name: &str) -> Option<usize> {
        self.storage
            .arrays
            .iter()
            .position(|array| array.get_name() == name)
    }

    fn should_copy(&self, name: &str) -> bool {
        match self.storage.copy_field_flags.get(name) {
            Some(copy) => *copy,
            None => self.storage.do_copy_all_on && !self.storage.do_copy_all_off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_array_replaces_by_name_and_returns_index() {
        let mut field_data = FieldData::new();
        assert_eq!(
            field_data.add_field_data_array(FieldDataArray::from_f64("temperature", 1, vec![1.0])),
            0
        );
        assert_eq!(
            field_data.add_field_data_array(FieldDataArray::from_f64(
                "temperature",
                1,
                vec![2.0, 3.0]
            )),
            0
        );

        assert_eq!(field_data.get_number_of_arrays(), 1);
        assert_eq!(
            field_data
                .get_field_data_array("temperature")
                .expect("array exists")
                .get_number_of_tuples(),
            2
        );
    }

    #[test]
    fn remove_array_compacts_order() {
        let mut field_data = FieldData::new();
        field_data.add_field_data_array(FieldDataArray::from_i64("a", 1, vec![1]));
        field_data.add_field_data_array(FieldDataArray::from_i64("b", 1, vec![2]));
        field_data.add_field_data_array(FieldDataArray::from_i64("c", 1, vec![3]));

        let removed = field_data
            .remove_field_data_array("b")
            .expect("array removed");

        assert_eq!(removed.get_name(), "b");
        assert_eq!(field_data.names(), vec!["a", "c"]);
        assert_eq!(field_data.get_array_name(1), Some("c"));
    }

    #[test]
    fn aggregate_component_and_tuple_counts_match_vtk_field_data() {
        let mut field_data = FieldData::new();
        assert_eq!(field_data.get_number_of_components(), 0);
        assert_eq!(field_data.get_number_of_tuples(), 0);

        field_data.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1, 2]));
        field_data.add_field_data_array(FieldDataArray::from_f64(
            "vectors",
            3,
            vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0],
        ));

        assert_eq!(field_data.get_number_of_components(), 4);
        assert_eq!(field_data.get_number_of_components(), 4);
        assert_eq!(field_data.get_number_of_tuples(), 2);
        assert_eq!(field_data.get_number_of_tuples(), 2);
    }

    #[test]
    fn copy_structure_copies_names_and_components_without_values() {
        let mut source = FieldData::new();
        source.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1, 2]));
        source.add_field_data_array(FieldDataArray::from_f64(
            "vectors",
            3,
            vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0],
        ));

        let mut structure = FieldData::new();
        structure.copy_structure(&source);

        assert_eq!(structure.names(), vec!["ids", "vectors"]);
        assert_eq!(
            structure
                .get_field_data_array("vectors")
                .expect("vectors")
                .get_number_of_components(),
            3
        );
        assert_eq!(structure.get_number_of_tuples(), 0);
        assert!(structure
            .iter()
            .all(|array| array.values_as_variants().is_empty()
                && !array.shares_values_with(
                    source
                        .get_field_data_array(array.get_name())
                        .expect("source array")
                )));
    }

    #[test]
    fn copy_structure_preserves_empty_array_data_type() {
        let mut source = FieldData::new();
        source.add_field_data_array(FieldDataArray::new_with_data_type(
            "empty_strings",
            1,
            VtkDataType::String,
        ));

        let mut structure = FieldData::new();
        structure.copy_structure(&source);

        let array = structure
            .get_field_data_array("empty_strings")
            .expect("array");
        assert_eq!(array.get_data_type(), VtkDataType::String);
        assert!(array.values_as_variants().is_empty());
    }

    #[test]
    fn field_data_mtime_includes_child_arrays() {
        let mut field_data = FieldData::new();
        let initial = field_data.get_m_time();
        field_data.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1]));
        let after_add = field_data.get_m_time();
        assert!(after_add > initial);

        field_data
            .get_array_mut("ids")
            .expect("ids")
            .set_number_of_tuples(3);

        assert!(field_data.get_m_time() > after_add);
    }

    #[test]
    fn actual_memory_size_sums_array_storage() {
        let mut field_data = FieldData::new();
        field_data.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1, 2, 3]));
        field_data.add_field_data_array(FieldDataArray::from_strings(
            "labels",
            vec![String::from("left"), String::from("right")],
        ));

        assert_eq!(
            field_data.get_actual_memory_size(),
            field_data
                .iter()
                .map(FieldDataArray::get_actual_memory_size)
                .sum()
        );
        assert!(field_data.get_actual_memory_size() >= 1);
    }

    #[test]
    fn initialize_clears_arrays_and_resets_copy_flags() {
        let mut field_data = FieldData::new();
        field_data.add_field_data_array(FieldDataArray::from_u8("ghosts", 1, vec![1]));
        field_data.copy_all_off();
        field_data.copy_field_on("keep");

        field_data.initialize();

        assert!(field_data.get_number_of_arrays() == 0);
        assert_eq!(field_data.get_flag("keep"), None);

        let source = FieldDataArray::from_i64("new", 1, vec![7]);
        let mut input = FieldData::new();
        input.add_field_data_array(source);
        field_data.pass_data(&input);
        assert!(field_data.has_array("new"));
    }

    #[test]
    fn allocate_reserves_slots_without_active_arrays() {
        let mut field_data = FieldData::new();

        field_data.allocate_arrays(4);

        assert_eq!(field_data.get_number_of_arrays(), 0);
        assert_eq!(field_data.reserved_array_slots(), 4);
    }

    #[test]
    fn reset_clears_tuples_without_releasing_array_capacity_and_squeeze_reclaims_it() {
        let mut field_data = FieldData::new();
        field_data.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1, 2, 3]));
        field_data
            .get_array_mut("ids")
            .expect("ids")
            .reserve_values(16);
        let reserved = field_data
            .get_field_data_array("ids")
            .expect("ids")
            .capacity();

        field_data.reset();

        let array = field_data.get_field_data_array("ids").expect("ids");
        assert_eq!(array.get_number_of_tuples(), 0);
        assert!(array.capacity() >= reserved);

        field_data.squeeze();
        assert_eq!(
            field_data
                .get_field_data_array("ids")
                .expect("ids")
                .capacity(),
            0
        );
    }

    #[test]
    fn copy_flags_control_pass_data() {
        let mut input = FieldData::new();
        input.add_field_data_array(FieldDataArray::from_i64("blocked", 1, vec![1]));
        input.add_field_data_array(FieldDataArray::from_i64("explicit", 1, vec![2]));

        let mut output = FieldData::new();
        output.copy_all_off();
        output.copy_field_on("explicit");
        output.pass_data(&input);

        assert!(!output.has_array("blocked"));
        assert!(output.has_array("explicit"));
    }

    #[test]
    fn copy_flags_from_copies_field_names_and_boolean_values() {
        let mut source = FieldData::new();
        source.copy_field_off("skip");
        source.copy_field_on("keep");

        let mut dest = FieldData::new();
        dest.copy_flags(&source);

        assert_eq!(dest.get_flag("skip"), Some(false));
        assert_eq!(dest.get_flag("keep"), Some(true));
    }

    #[test]
    fn set_insert_next_tuple_copy_all_arrays_by_order() {
        let mut source = FieldData::new();
        source.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![10, 20, 30]));
        source.add_field_data_array(FieldDataArray::from_f64(
            "vectors",
            3,
            vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0],
        ));

        let mut dest = FieldData::new();
        dest.copy_structure(&source);
        dest.set_number_of_tuples(2);

        dest.set_tuple(0, 2, &source);
        dest.insert_tuple(1, 1, &source);
        assert_eq!(dest.insert_next_tuple(0, &source), 2);

        assert_eq!(
            dest.get_field_data_array("ids")
                .expect("ids")
                .values_as_variants(),
            &[Variant::I64(30), Variant::I64(20), Variant::I64(10)]
        );
        assert_eq!(
            dest.get_field_data_array("vectors")
                .expect("vectors")
                .values_as_variants(),
            &[
                Variant::F64(3.0),
                Variant::F64(0.0),
                Variant::F64(0.0),
                Variant::F64(2.0),
                Variant::F64(0.0),
                Variant::F64(0.0),
                Variant::F64(1.0),
                Variant::F64(0.0),
                Variant::F64(0.0)
            ]
        );
    }

    #[test]
    fn get_field_copies_selected_tuple_order_and_component_lookup() {
        let mut source = FieldData::new();
        source.add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![10, 20, 30]));
        source.add_field_data_array(FieldDataArray::from_f64(
            "vectors",
            3,
            vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0],
        ));

        let mut array_component = -1;
        assert_eq!(
            source.get_array_containing_component(0, &mut array_component),
            0
        );
        assert_eq!(array_component, 0);
        assert_eq!(
            source.get_array_containing_component(2, &mut array_component),
            1
        );
        assert_eq!(array_component, 1);
        assert_eq!(
            source.get_array_containing_component(4, &mut array_component),
            -1
        );

        let mut field = FieldData::new();
        field.copy_structure(&source);
        source.get_field(&[2, 0], &mut field);

        assert_eq!(field.names(), vec!["ids", "vectors"]);
        assert_eq!(
            field
                .get_field_data_array("ids")
                .expect("ids")
                .values_as_variants(),
            &[Variant::I64(30), Variant::I64(10)]
        );
        assert_eq!(
            field
                .get_field_data_array("vectors")
                .expect("vectors")
                .values_as_variants(),
            &[
                Variant::F64(3.0),
                Variant::F64(0.0),
                Variant::F64(0.0),
                Variant::F64(1.0),
                Variant::F64(0.0),
                Variant::F64(0.0)
            ]
        );
    }

    #[test]
    fn deep_copy_duplicates_storage_and_shallow_copy_shares_storage() {
        let mut input = FieldData::new();
        input.add_field_data_array(FieldDataArray::from_f64("coords", 3, vec![1.0, 2.0, 3.0]));

        let mut deep = FieldData::new();
        deep.deep_copy(&input);
        let mut shallow = FieldData::new();
        shallow.shallow_copy(&input);

        assert!(shallow.shares_storage_with(&input));
        let input_array = input
            .get_field_data_array("coords")
            .expect("input array exists");
        assert!(!deep
            .get_field_data_array("coords")
            .expect("deep copy exists")
            .shares_values_with(input_array));
        assert!(shallow
            .get_field_data_array("coords")
            .expect("shallow copy exists")
            .shares_values_with(input_array));

        shallow
            .get_array_mut("coords")
            .expect("shallow copy exists")
            .set_value(3, Variant::F64(4.0));
        assert!(!shallow.shares_storage_with(&input));
        assert_eq!(input_array.values_as_variants().len(), 3);
    }

    #[test]
    fn set_number_of_tuples_resizes_values_with_cow() {
        let input = FieldDataArray::from_i64("ids", 1, vec![1, 2]);
        let mut shallow = input.shallow_clone();

        shallow.set_number_of_tuples(3);

        assert!(!shallow.shares_values_with(&input));
        assert_eq!(
            shallow.values_as_variants(),
            &[Variant::I64(1), Variant::I64(2), Variant::I64(0)]
        );
        assert_eq!(
            input.values_as_variants(),
            &[Variant::I64(1), Variant::I64(2)]
        );
    }
}
