use super::{
    data_array::{
        component_count_to_usize, id_count_to_usize, int_index_to_usize, vtk_id_to_usize,
    },
    vtk_type::{VtkDataType, VtkIdType},
};
use std::sync::Arc;

fn id_count_from_usize(count: usize) -> VtkIdType {
    VtkIdType::try_from(count).expect("usize count must fit vtkIdType")
}

fn byte_len_for_bits(bits: usize) -> usize {
    bits.div_ceil(8)
}

fn bit_mask(bit_id: usize) -> u8 {
    0x80 >> (bit_id % 8)
}

fn bit_value(value: impl Into<f64>) -> u8 {
    (value.into() as i32 != 0) as u8
}

/// Packed VTK `vtkBitArray`.
///
/// VTK origin: `VTK/Common/Core/vtkBitArray.h` and
/// `VTK/Common/Core/vtkBitArray.cxx`.
#[derive(Debug, Clone)]
pub struct BitArray {
    storage: Arc<BitArrayStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct BitArrayStorage {
    name: String,
    number_of_components: usize,
    number_of_values: usize,
    capacity_bits: usize,
    buffer: Vec<u8>,
    component_names: Vec<Option<String>>,
    modified_time: u64,
}

impl PartialEq for BitArray {
    fn eq(&self, other: &Self) -> bool {
        self.storage.name == other.storage.name
            && self.storage.number_of_components == other.storage.number_of_components
            && self.storage.number_of_values == other.storage.number_of_values
            && (0..self.storage.number_of_values)
                .all(|idx| self.get_value(idx as VtkIdType) == other.get_value(idx as VtkIdType))
            && self.storage.component_names == other.storage.component_names
    }
}

impl Default for BitArray {
    fn default() -> Self {
        Self::new()
    }
}

impl BitArray {
    /// VTK: `vtkBitArray::New`.
    pub fn new() -> Self {
        Self::with_name_and_number_of_components("", 1)
    }

    pub(crate) fn with_name_and_number_of_components(
        name: impl Into<String>,
        number_of_components: usize,
    ) -> Self {
        assert!(
            number_of_components > 0,
            "number_of_components must be greater than zero"
        );
        Self {
            storage: Arc::new(BitArrayStorage {
                name: name.into(),
                number_of_components,
                number_of_values: 0,
                capacity_bits: 0,
                buffer: Vec::new(),
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_vec(
        name: impl Into<String>,
        values: Vec<u8>,
        number_of_components: usize,
    ) -> Self {
        let mut array = Self::with_name_and_number_of_components(name, number_of_components);
        array.set_number_of_values(values.len() as VtkIdType);
        for (idx, value) in values.into_iter().enumerate() {
            array.set_value(idx as VtkIdType, value as i32);
        }
        array
    }

    fn storage_mut(&mut self) -> &mut BitArrayStorage {
        Arc::make_mut(&mut self.storage)
    }

    fn modified(&mut self) {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    fn ensure_capacity_bits(&mut self, capacity_bits: usize) {
        if capacity_bits <= self.storage.capacity_bits {
            return;
        }
        let storage = self.storage_mut();
        storage.capacity_bits = capacity_bits;
        storage.buffer.resize(byte_len_for_bits(capacity_bits), 0);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    fn initialize_unused_bits_in_last_byte(&mut self) {
        let number_of_values = self.storage.number_of_values;
        if number_of_values == 0 || number_of_values % 8 == 0 {
            return;
        }
        let byte_idx = number_of_values / 8;
        let used_bits = number_of_values % 8;
        let keep_mask = 0xff << (8 - used_bits);
        let storage = self.storage_mut();
        if byte_idx < storage.buffer.len() {
            storage.buffer[byte_idx] &= keep_mask;
        }
    }

    /// VTK: `vtkAbstractArray::GetName`.
    pub fn get_name(&self) -> &str {
        &self.storage.name
    }

    /// VTK: `vtkAbstractArray::SetName`.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let storage = self.storage_mut();
        storage.name = name.into();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkBitArray::GetDataType`.
    pub fn get_data_type(&self) -> VtkDataType {
        VtkDataType::Bit
    }

    /// VTK: `vtkBitArray::GetDataTypeSize`.
    pub fn get_data_type_size(&self) -> i32 {
        0
    }

    pub fn get_data_type_range(&self) -> [f64; 2] {
        [0.0, 1.0]
    }

    pub fn get_data_type_min(&self) -> f64 {
        0.0
    }

    pub fn get_data_type_max(&self) -> f64 {
        1.0
    }

    /// VTK: `vtkAbstractArray::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.storage.number_of_components as i32
    }

    /// VTK: `vtkAbstractArray::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        let number_of_components = component_count_to_usize(number_of_components);
        let storage = self.storage_mut();
        storage.number_of_components = number_of_components;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        id_count_from_usize(self.storage.number_of_values / self.storage.number_of_components)
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        self.set_number_of_values(id_count_from_usize(
            id_count_to_usize(number_of_tuples) * self.storage.number_of_components,
        ));
    }

    /// VTK: `vtkAbstractArray::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> VtkIdType {
        id_count_from_usize(self.storage.number_of_values)
    }

    /// VTK: `vtkBitArray::SetNumberOfValues`.
    pub fn set_number_of_values(&mut self, number_of_values: VtkIdType) -> bool {
        let number_of_values = id_count_to_usize(number_of_values);
        self.ensure_capacity_bits(number_of_values);
        let storage = self.storage_mut();
        storage.number_of_values = number_of_values;
        storage.modified_time = storage.modified_time.saturating_add(1);
        self.initialize_unused_bits_in_last_byte();
        true
    }

    #[allow(dead_code)]
    pub(crate) fn capacity(&self) -> usize {
        self.storage.capacity_bits
    }

    /// VTK: `vtkBitArray::ReserveTuples`.
    pub fn reserve_tuples(&mut self, number_of_tuples: VtkIdType) -> bool {
        let requested = id_count_to_usize(number_of_tuples) * self.storage.number_of_components;
        if requested > self.storage.capacity_bits {
            let current_tuples = self.storage.capacity_bits / self.storage.number_of_components;
            self.ensure_capacity_bits(
                (current_tuples + id_count_to_usize(number_of_tuples))
                    * self.storage.number_of_components,
            );
        }
        true
    }

    /// VTK: `vtkAbstractArray::ReserveValues`.
    pub fn reserve_values(&mut self, number_of_values: VtkIdType) -> bool {
        self.ensure_capacity_bits(id_count_to_usize(number_of_values));
        true
    }

    /// VTK: `vtkAbstractArray::Allocate`.
    pub fn allocate(&mut self, number_of_values: VtkIdType) -> bool {
        self.initialize();
        self.reserve_values(number_of_values)
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.storage.number_of_values == 0
    }

    /// VTK: `vtkAbstractArray::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.number_of_values = 0;
        storage.capacity_bits = 0;
        storage.buffer.clear();
        storage.buffer.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::Reset`.
    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.number_of_values = 0;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkBitArray::Squeeze`.
    pub fn squeeze(&mut self) {
        if self.storage.capacity_bits > self.storage.number_of_values {
            let number_of_values = self.storage.number_of_values;
            let storage = self.storage_mut();
            storage.buffer.truncate(byte_len_for_bits(number_of_values));
            storage.buffer.shrink_to_fit();
            storage.capacity_bits = number_of_values;
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
        self.initialize_unused_bits_in_last_byte();
    }

    /// VTK: `vtkBitArray::GetTuple`.
    pub fn get_tuple(&self, tuple_idx: VtkIdType) -> Vec<f64> {
        self.get_typed_tuple(tuple_idx)
            .into_iter()
            .map(f64::from)
            .collect()
    }

    /// VTK: `vtkBitArray::GetTypedTuple`.
    pub fn get_typed_tuple(&self, tuple_idx: VtkIdType) -> Vec<u8> {
        let tuple_idx = vtk_id_to_usize(tuple_idx);
        let start = tuple_idx * self.storage.number_of_components;
        assert!(
            start + self.storage.number_of_components <= self.storage.number_of_values,
            "tuple index out of range"
        );
        (0..self.storage.number_of_components)
            .map(|component| self.get_value((start + component) as VtkIdType) as u8)
            .collect()
    }

    /// VTK: `vtkBitArray::SetTypedTuple`.
    pub fn set_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[u8]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        let start = vtk_id_to_usize(tuple_idx) * self.storage.number_of_components;
        assert!(
            start + tuple.len() <= self.storage.number_of_values,
            "tuple index out of range"
        );
        for (offset, &value) in tuple.iter().enumerate() {
            self.set_value((start + offset) as VtkIdType, value as i32);
        }
    }

    pub(crate) fn set_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        let converted: Vec<_> = tuple.iter().map(|&value| bit_value(value)).collect();
        self.set_typed_tuple(tuple_idx as VtkIdType, &converted);
    }

    pub fn set_tuple(&mut self, dst_tuple_idx: VtkIdType, src_tuple_idx: VtkIdType, source: &Self) {
        let tuple = source.get_typed_tuple(src_tuple_idx);
        self.set_typed_tuple(dst_tuple_idx, &tuple);
    }

    pub fn insert_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[u8]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        let start = vtk_id_to_usize(tuple_idx) * self.storage.number_of_components;
        for (offset, &value) in tuple.iter().enumerate() {
            self.insert_value((start + offset) as VtkIdType, value as i32);
        }
    }

    pub(crate) fn insert_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        let converted: Vec<_> = tuple.iter().map(|&value| bit_value(value)).collect();
        self.insert_typed_tuple(tuple_idx as VtkIdType, &converted);
    }

    pub fn insert_tuple(
        &mut self,
        dst_tuple_idx: VtkIdType,
        src_tuple_idx: VtkIdType,
        source: &Self,
    ) {
        let tuple = source.get_typed_tuple(src_tuple_idx);
        self.insert_typed_tuple(dst_tuple_idx, &tuple);
    }

    pub fn insert_next_typed_tuple(&mut self, tuple: &[u8]) -> VtkIdType {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_typed_tuple(tuple_idx, tuple);
        tuple_idx
    }

    pub fn insert_next_tuple(&mut self, src_tuple_idx: VtkIdType, source: &Self) -> VtkIdType {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_tuple(tuple_idx, src_tuple_idx, source);
        tuple_idx
    }

    /// VTK: `vtkBitArray::GetComponent`.
    pub fn get_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> f64 {
        f64::from(self.get_typed_component(tuple_idx, component_idx))
    }

    /// VTK: `vtkBitArray::SetComponent`.
    pub fn set_component(&mut self, tuple_idx: VtkIdType, component_idx: i32, value: f64) {
        let value_idx = vtk_id_to_usize(tuple_idx) * self.storage.number_of_components
            + int_index_to_usize(component_idx);
        self.set_value(value_idx as VtkIdType, bit_value(value) as i32);
    }

    /// VTK: `vtkBitArray::InsertComponent`.
    pub fn insert_component(&mut self, tuple_idx: VtkIdType, component_idx: i32, value: f64) {
        let value_idx = vtk_id_to_usize(tuple_idx) * self.storage.number_of_components
            + int_index_to_usize(component_idx);
        self.insert_value(value_idx as VtkIdType, bit_value(value) as i32);
    }

    pub fn fill_component(&mut self, component_idx: i32, value: f64) {
        let component_idx = int_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        let value = bit_value(value) as i32;
        for tuple_idx in 0..vtk_id_to_usize(self.get_number_of_tuples()) {
            self.set_value(
                (tuple_idx * self.storage.number_of_components + component_idx) as VtkIdType,
                value,
            );
        }
    }

    pub fn fill(&mut self, value: f64) {
        let value = bit_value(value) as i32;
        for value_idx in 0..self.storage.number_of_values {
            self.set_value(value_idx as VtkIdType, value);
        }
    }

    /// VTK: `vtkAbstractArray::GetTuples(tupleIds, output)`.
    pub fn get_tuples(&self, tuple_ids: &[VtkIdType], output: &mut Self) {
        output.set_number_of_components(self.get_number_of_components());
        output.set_number_of_tuples(tuple_ids.len() as VtkIdType);
        for (dst_tuple_idx, &src_tuple_idx) in tuple_ids.iter().enumerate() {
            let tuple = self.get_typed_tuple(src_tuple_idx);
            output.set_typed_tuple(dst_tuple_idx as VtkIdType, &tuple);
        }
        output.storage_mut().component_names = self.storage.component_names.clone();
    }

    #[cfg(test)]
    pub(crate) fn get_tuples_in_range(&self, first: usize, last_inclusive: usize) -> Self {
        assert!(first <= last_inclusive, "first tuple must be <= last tuple");
        let mut output = Self::with_name_and_number_of_components(
            self.storage.name.clone(),
            self.storage.number_of_components,
        );
        output.set_number_of_tuples((last_inclusive - first + 1) as VtkIdType);
        for (dst_tuple_idx, src_tuple_idx) in (first..=last_inclusive).enumerate() {
            let tuple = self.get_typed_tuple(src_tuple_idx as VtkIdType);
            output.set_typed_tuple(dst_tuple_idx as VtkIdType, &tuple);
        }
        output
    }

    /// VTK: `vtkBitArray::InsertTuples(dstStart, n, srcStart, source)`.
    pub fn insert_tuples(
        &mut self,
        dst_start: VtkIdType,
        count: VtkIdType,
        src_start: VtkIdType,
        source: &Self,
    ) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        if count == 0 {
            return;
        }
        assert!(
            src_start + count <= source.get_number_of_tuples(),
            "source tuple index out of range"
        );
        for offset in 0..count {
            self.insert_tuple(dst_start + offset, src_start + offset, source);
        }
    }

    pub fn set_component_name(&mut self, component: VtkIdType, name: impl Into<String>) {
        let component = vtk_id_to_usize(component);
        let storage = self.storage_mut();
        if component >= storage.component_names.len() {
            storage.component_names.resize_with(component + 1, || None);
        }
        storage.component_names[component] = Some(name.into());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_component_name(&self, component: VtkIdType) -> Option<&str> {
        self.storage
            .component_names
            .get(vtk_id_to_usize(component))
            .and_then(|name| name.as_deref())
    }

    pub(crate) fn has_a_component_name(&self) -> bool {
        !self.storage.component_names.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn copy_component_names_from(&mut self, other: &Self) -> bool {
        let storage = self.storage_mut();
        storage
            .component_names
            .clone_from(&other.storage.component_names);
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    /// VTK: `vtkDataArray::GetData`.
    pub fn get_data(
        &self,
        tuple_min: VtkIdType,
        tuple_max: VtkIdType,
        component_min: i32,
        component_max: i32,
    ) -> Vec<f64> {
        assert!(tuple_min <= tuple_max, "tuple_min must be <= tuple_max");
        assert!(
            component_min <= component_max,
            "component_min must be <= component_max"
        );
        let mut output = Vec::with_capacity(
            id_count_to_usize(tuple_max - tuple_min + 1)
                * (int_index_to_usize(component_max) - int_index_to_usize(component_min) + 1),
        );
        for tuple_idx in tuple_min..=tuple_max {
            for component_idx in component_min..=component_max {
                output.push(self.get_component(tuple_idx, component_idx));
            }
        }
        output
    }

    pub(crate) fn tuple_as_f64(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple(tuple_idx as VtkIdType)
    }

    pub(crate) fn checked_tuple_as_f64(
        &self,
        tuple_idx: usize,
    ) -> Result<Vec<f64>, crate::common::core::ArrayError> {
        let number_of_tuples = vtk_id_to_usize(self.get_number_of_tuples());
        if tuple_idx >= number_of_tuples {
            return Err(crate::common::core::ArrayError::TupleOutOfRange {
                tuple: tuple_idx,
                number_of_tuples,
            });
        }
        Ok(self.tuple_as_f64(tuple_idx))
    }

    pub fn get_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
        self.compute_range(component as isize)
    }

    pub fn get_finite_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
        self.compute_range(component as isize)
    }

    pub fn get_range(&self) -> Option<[f64; 2]> {
        self.get_range_with_component(0)
    }

    fn compute_range(&self, mut component: isize) -> Option<[f64; 2]> {
        if self.get_number_of_tuples() == 0 {
            return None;
        }
        if component >= self.storage.number_of_components as isize {
            return None;
        }
        if component < 0 && self.storage.number_of_components == 1 {
            component = 0;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for tuple_idx in 0..vtk_id_to_usize(self.get_number_of_tuples()) {
            let value = if component < 0 {
                self.get_typed_tuple(tuple_idx as VtkIdType)
                    .into_iter()
                    .map(|value| f64::from(value).powi(2))
                    .sum::<f64>()
                    .sqrt()
            } else {
                f64::from(self.get_typed_component(tuple_idx as VtkIdType, component as i32))
            };
            min = min.min(value);
            max = max.max(value);
        }
        Some([min, max])
    }

    pub fn get_max_norm(&self) -> f64 {
        self.compute_range(-1).map_or(0.0, |range| range[1])
    }

    /// VTK: `vtkBitArray::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.buffer.capacity().div_ceil(1024)
    }

    /// VTK: `vtkBitArray::GetVoidPointer` / `GetBuffer`.
    #[allow(dead_code)]
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.storage.buffer
    }

    /// Mutable Rust equivalent of VTK raw packed buffer access.
    #[allow(dead_code)]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        self.storage_mut().buffer.as_mut_slice()
    }

    /// VTK: `vtkBitArray::GetTypedComponent`.
    pub fn get_typed_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> u8 {
        let component_idx = int_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        self.get_value(
            (vtk_id_to_usize(tuple_idx) * self.storage.number_of_components + component_idx)
                as VtkIdType,
        ) as u8
    }

    /// VTK: `vtkBitArray::SetTypedComponent`.
    pub fn set_typed_component(&mut self, tuple_idx: VtkIdType, component_idx: i32, value: u8) {
        let component_idx = int_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        let value_idx =
            vtk_id_to_usize(tuple_idx) * self.storage.number_of_components + component_idx;
        self.set_value(value_idx as VtkIdType, value as i32);
    }

    /// VTK: `vtkBitArray::GetValue`.
    pub fn get_value(&self, value_idx: VtkIdType) -> i32 {
        let value_idx = vtk_id_to_usize(value_idx);
        assert!(
            value_idx < self.storage.number_of_values,
            "value index out of range"
        );
        let byte_idx = value_idx / 8;
        ((self.storage.buffer[byte_idx] & bit_mask(value_idx)) != 0) as i32
    }

    /// VTK: `vtkBitArray::SetValue`.
    pub fn set_value(&mut self, value_idx: VtkIdType, value: i32) {
        let value_idx = vtk_id_to_usize(value_idx);
        assert!(
            value_idx < self.storage.number_of_values,
            "value index out of range"
        );
        let byte_idx = value_idx / 8;
        let mask = bit_mask(value_idx);
        let storage = self.storage_mut();
        if value != 0 {
            storage.buffer[byte_idx] |= mask;
        } else {
            storage.buffer[byte_idx] &= !mask;
        }
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkBitArray::InsertValue`.
    pub fn insert_value(&mut self, value_idx: VtkIdType, value: i32) {
        let value_idx = vtk_id_to_usize(value_idx);
        if value_idx >= self.storage.capacity_bits {
            self.reserve_tuples(id_count_from_usize(
                (value_idx + 1) / self.storage.number_of_components + 1,
            ));
        }
        if value_idx >= self.storage.number_of_values {
            self.storage_mut().number_of_values = value_idx + 1;
        }
        self.set_value(value_idx as VtkIdType, value);
        self.initialize_unused_bits_in_last_byte();
    }

    pub fn insert_next_value(&mut self, value: i32) -> VtkIdType {
        let value_idx = self.get_number_of_values();
        self.insert_value(value_idx, value);
        value_idx
    }

    pub fn lookup_value(&self, value: i32) -> VtkIdType {
        let value = (value != 0) as i32;
        (0..self.storage.number_of_values)
            .find(|&idx| self.get_value(idx as VtkIdType) == value)
            .map_or(-1, id_count_from_usize)
    }

    pub fn lookup_value_ids(&self, value: i32) -> Vec<VtkIdType> {
        let value = (value != 0) as i32;
        (0..self.storage.number_of_values)
            .filter(|&idx| self.get_value(idx as VtkIdType) == value)
            .map(id_count_from_usize)
            .collect()
    }

    pub fn data_changed(&mut self) {
        self.modified();
    }

    pub fn clear_lookup(&mut self) {}

    pub fn remove_tuple(&mut self, tuple_idx: VtkIdType) {
        if tuple_idx < 0 || tuple_idx >= self.get_number_of_tuples() {
            return;
        }
        if tuple_idx == self.get_number_of_tuples() - 1 {
            let new_values = self
                .storage
                .number_of_values
                .saturating_sub(self.storage.number_of_components);
            self.set_number_of_values(new_values as VtkIdType);
        }
    }

    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new((*other.storage).clone());
        self.modified();
    }

    pub fn shallow_copy(&mut self, other: &Self) {
        self.storage = Arc::clone(&other.storage);
    }

    pub(crate) fn deep_clone(&self) -> Self {
        let mut output = Self::with_name_and_number_of_components(
            self.get_name(),
            self.storage.number_of_components,
        );
        output.deep_copy(self);
        output
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }

    pub fn get_m_time(&self) -> u64 {
        self.storage.modified_time
    }
}
