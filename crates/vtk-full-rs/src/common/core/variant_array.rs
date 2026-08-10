use super::vtk_type::{VtkDataType, VtkIdType};
use std::{mem, sync::Arc};

fn component_count_to_usize(number_of_components: i32) -> usize {
    usize::try_from(number_of_components.max(1)).expect("component count must fit usize")
}

fn component_index_to_usize(component: i32) -> usize {
    usize::try_from(component).expect("component index must be non-negative")
}

fn id_count_to_usize(count: VtkIdType) -> usize {
    usize::try_from(count.max(0)).expect("vtkIdType count must fit usize")
}

fn id_index_to_usize(index: VtkIdType) -> usize {
    usize::try_from(index).expect("vtkIdType index must be non-negative")
}

fn id_count_from_usize(count: usize) -> VtkIdType {
    VtkIdType::try_from(count).expect("usize count must fit vtkIdType")
}

/// VTK: `vtkVariant`.
#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    Invalid,
    F64(f64),
    I64(i64),
    U8(u8),
    String(String),
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid => f.write_str("(invalid)"),
            Self::F64(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::U8(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariantArray {
    storage: Arc<VariantArrayStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct VariantArrayStorage {
    name: String,
    number_of_components: usize,
    values: Vec<Variant>,
    component_names: Vec<Option<String>>,
    modified_time: u64,
}

impl PartialEq for VariantArray {
    fn eq(&self, other: &Self) -> bool {
        self.storage.name == other.storage.name
            && self.storage.number_of_components == other.storage.number_of_components
            && self.storage.values == other.storage.values
            && self.storage.component_names == other.storage.component_names
    }
}

impl VariantArray {
    const DEFAULT_VALUE: Variant = Variant::Invalid;

    pub fn new() -> Self {
        Self::with_name_and_number_of_components("", 1)
    }

    pub fn extended_new() -> Self {
        Self::new()
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
            storage: Arc::new(VariantArrayStorage {
                name: name.into(),
                number_of_components,
                values: Vec::new(),
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_values(
        name: impl Into<String>,
        values: Vec<Variant>,
        number_of_components: usize,
    ) -> Self {
        assert!(
            number_of_components > 0,
            "number_of_components must be greater than zero"
        );
        assert!(
            values.len() % number_of_components == 0,
            "value count must be divisible by number_of_components"
        );
        Self {
            storage: Arc::new(VariantArrayStorage {
                name: name.into(),
                number_of_components,
                values,
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    fn storage_mut(&mut self) -> &mut VariantArrayStorage {
        Arc::make_mut(&mut self.storage)
    }

    pub fn get_name(&self) -> &str {
        &self.storage.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let storage = self.storage_mut();
        storage.name = name.into();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_number_of_components(&self) -> i32 {
        self.storage.number_of_components as i32
    }

    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        let number_of_components = component_count_to_usize(number_of_components);
        let storage = self.storage_mut();
        storage.number_of_components = number_of_components;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_number_of_tuples(&self) -> VtkIdType {
        id_count_from_usize(self.storage.values.len() / self.storage.number_of_components)
    }

    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let storage = self.storage_mut();
        storage.values.resize(
            id_count_to_usize(number_of_tuples) * storage.number_of_components,
            Self::DEFAULT_VALUE,
        );
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_number_of_values(&self) -> VtkIdType {
        id_count_from_usize(self.storage.values.len())
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.storage.values.capacity()
    }

    pub fn set_number_of_values(&mut self, number_of_values: VtkIdType) -> bool {
        let storage = self.storage_mut();
        storage
            .values
            .resize(id_count_to_usize(number_of_values), Self::DEFAULT_VALUE);
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    pub fn reserve_tuples(&mut self, number_of_tuples: VtkIdType) -> bool {
        let values = id_count_to_usize(number_of_tuples) * self.storage.number_of_components;
        if values > self.storage.values.capacity() {
            let storage = self.storage_mut();
            storage.values.reserve(values - storage.values.capacity());
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
        true
    }

    pub fn reserve_values(&mut self, number_of_values: VtkIdType) -> bool {
        let number_of_values = id_count_to_usize(number_of_values);
        if number_of_values > self.storage.values.capacity() {
            let storage = self.storage_mut();
            storage
                .values
                .reserve(number_of_values - storage.values.capacity());
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
        true
    }

    pub fn allocate(&mut self, number_of_values: VtkIdType) -> bool {
        self.initialize();
        let storage = self.storage_mut();
        storage.values.reserve(id_count_to_usize(number_of_values));
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    pub(crate) fn as_slice(&self) -> &[Variant] {
        &self.storage.values
    }

    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.values.clear();
        storage.values.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.values.clear();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.values.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkVariantArray::GetDataType`.
    pub fn get_data_type(&self) -> VtkDataType {
        VtkDataType::Variant
    }

    /// VTK: `vtkVariantArray::GetDataTypeSize`.
    pub fn get_data_type_size(&self) -> i32 {
        mem::size_of::<Variant>() as i32
    }

    /// VTK: `vtkVariantArray::GetElementComponentSize`.
    pub fn get_element_component_size(&self) -> i32 {
        self.get_data_type_size()
    }

    /// VTK: `vtkVariantArray::IsNumeric`.
    pub fn is_numeric(&self) -> i32 {
        0
    }

    /// VTK: `vtkVariantArray::CopyComponent`.
    pub fn copy_component(
        &mut self,
        dst_component: i32,
        source: &Self,
        src_component: i32,
    ) -> bool {
        if source.get_number_of_tuples() != self.get_number_of_tuples()
            || src_component < 0
            || src_component >= source.get_number_of_components()
            || dst_component < 0
            || dst_component >= self.get_number_of_components()
        {
            return false;
        }

        let dst_component = component_index_to_usize(dst_component);
        let src_component = component_index_to_usize(src_component);
        let dst_components = self.storage.number_of_components;
        let src_components = source.storage.number_of_components;
        for tuple_idx in 0..id_count_to_usize(self.get_number_of_tuples()) {
            self.set_value(
                id_count_from_usize(tuple_idx * dst_components + dst_component),
                source.storage.values[tuple_idx * src_components + src_component].clone(),
            );
        }
        true
    }

    pub fn get_value(&self, value_idx: VtkIdType) -> &Variant {
        &self.storage.values[id_index_to_usize(value_idx)]
    }

    pub fn set_value(&mut self, value_idx: VtkIdType, value: Variant) {
        let value_idx = id_index_to_usize(value_idx);
        assert!(
            value_idx < self.storage.values.len(),
            "value index out of range"
        );
        let storage = self.storage_mut();
        storage.values[value_idx] = value;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn insert_value(&mut self, value_idx: VtkIdType, value: Variant) {
        if value_idx >= self.get_number_of_values() {
            self.set_number_of_values(value_idx + 1);
        }
        self.set_value(value_idx, value);
    }

    pub fn insert_next_value(&mut self, value: Variant) -> VtkIdType {
        let value_idx = self.get_number_of_values();
        let storage = self.storage_mut();
        storage.values.push(value);
        storage.modified_time = storage.modified_time.saturating_add(1);
        value_idx
    }

    pub fn get_typed_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> &Variant {
        let component_idx = component_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        self.get_value(id_count_from_usize(
            id_index_to_usize(tuple_idx) * self.storage.number_of_components + component_idx,
        ))
    }

    pub fn set_typed_component(
        &mut self,
        tuple_idx: VtkIdType,
        component_idx: i32,
        value: Variant,
    ) {
        let tuple_idx = id_index_to_usize(tuple_idx);
        let component_idx = component_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        self.set_value(
            id_count_from_usize(tuple_idx * self.storage.number_of_components + component_idx),
            value,
        );
    }

    pub fn get_typed_tuple(&self, tuple_idx: VtkIdType) -> &[Variant] {
        let tuple_idx = id_index_to_usize(tuple_idx);
        let start = tuple_idx * self.storage.number_of_components;
        let end = start + self.storage.number_of_components;
        self.storage
            .values
            .get(start..end)
            .expect("tuple index out of range")
    }

    pub fn set_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[Variant]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        let tuple_idx = id_index_to_usize(tuple_idx);
        let start = tuple_idx * self.storage.number_of_components;
        let end = start + self.storage.number_of_components;
        assert!(end <= self.storage.values.len(), "tuple index out of range");
        let storage = self.storage_mut();
        storage.values[start..end].clone_from_slice(tuple);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub(crate) fn insert_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[Variant]) {
        if tuple_idx >= self.get_number_of_tuples() {
            self.set_number_of_tuples(tuple_idx + 1);
        }
        self.set_typed_tuple(tuple_idx, tuple);
    }

    pub fn set_tuple(&mut self, dst_tuple_idx: VtkIdType, src_tuple_idx: VtkIdType, source: &Self) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        self.set_typed_tuple(dst_tuple_idx, source.get_typed_tuple(src_tuple_idx));
    }

    pub fn insert_tuple(
        &mut self,
        dst_tuple_idx: VtkIdType,
        src_tuple_idx: VtkIdType,
        source: &Self,
    ) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        self.insert_typed_tuple(dst_tuple_idx, source.get_typed_tuple(src_tuple_idx));
    }

    pub fn insert_next_tuple(&mut self, src_tuple_idx: VtkIdType, source: &Self) -> VtkIdType {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_tuple(tuple_idx, src_tuple_idx, source);
        tuple_idx
    }

    pub fn insert_tuples(
        &mut self,
        dst_start: VtkIdType,
        count: VtkIdType,
        src_start: VtkIdType,
        source: &Self,
    ) {
        for offset in 0..count.max(0) {
            self.insert_tuple(dst_start + offset, src_start + offset, source);
        }
    }

    pub fn interpolate_tuple_from(
        &mut self,
        source: &Self,
        source_tuples: &[VtkIdType],
        weights: &[f64],
        to_tuple: VtkIdType,
    ) -> bool {
        if source.get_number_of_components() != self.get_number_of_components()
            || source_tuples.is_empty()
            || source_tuples.len() != weights.len()
        {
            return false;
        }
        let max_weight_index = weights
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
            .expect("non-empty weights");
        self.insert_tuple(to_tuple, source_tuples[max_weight_index], source);
        true
    }

    pub fn interpolate_tuple_between(
        &mut self,
        source1: &Self,
        id1: VtkIdType,
        source2: &Self,
        id2: VtkIdType,
        t: f64,
        to_tuple: VtkIdType,
    ) -> bool {
        if source1.get_number_of_components() != self.get_number_of_components()
            || source2.get_number_of_components() != self.get_number_of_components()
        {
            return false;
        }
        if t >= 0.5 {
            self.insert_tuple(to_tuple, id2, source2);
        } else {
            self.insert_tuple(to_tuple, id1, source1);
        }
        true
    }

    pub fn set_component_name(&mut self, component: VtkIdType, name: impl Into<String>) {
        let component = id_index_to_usize(component);
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
            .get(id_index_to_usize(component))
            .and_then(|name| name.as_deref())
    }

    pub(crate) fn has_a_component_name(&self) -> bool {
        !self.storage.component_names.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn copy_component_names_from(&mut self, other: &Self) {
        let storage = self.storage_mut();
        storage
            .component_names
            .clone_from(&other.storage.component_names);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_actual_memory_size(&self) -> usize {
        (self.storage.values.capacity() * mem::size_of::<Variant>()).div_ceil(1024)
    }

    pub fn lookup_value(&self, value: Variant) -> VtkIdType {
        self.storage
            .values
            .iter()
            .position(|candidate| candidate == &value)
            .map_or(-1, id_count_from_usize)
    }

    pub fn lookup_value_ids(&self, value: Variant) -> Vec<VtkIdType> {
        self.storage
            .values
            .iter()
            .enumerate()
            .filter_map(|(idx, candidate)| (candidate == &value).then(|| id_count_from_usize(idx)))
            .collect()
    }

    /// VTK: `vtkVariantArray::DataChanged`.
    pub fn data_changed(&mut self) {
        self.modified();
    }

    /// VTK: `vtkVariantArray::DataElementChanged`.
    pub fn data_element_changed(&mut self, _id: VtkIdType) {
        self.modified();
    }

    /// VTK: `vtkVariantArray::ClearLookup`.
    pub fn clear_lookup(&mut self) {}

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
            component_count_to_usize(self.get_number_of_components()),
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

    pub fn modified(&mut self) {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_m_time(&self) -> u64 {
        self.storage.modified_time
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [Variant] {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
        &mut storage.values
    }
}
