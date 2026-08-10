use super::vtk_type::VtkIdType;
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

/// Shared string tuple array with VTK-shaped indexing.
///
/// VTK origin: audited basics from `VTK/Common/Core/vtkStringArray.cxx`.
#[derive(Debug, Clone)]
pub struct StringArray {
    storage: Arc<StringArrayStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct StringArrayStorage {
    name: String,
    number_of_components: usize,
    values: Vec<String>,
    component_names: Vec<Option<String>>,
    modified_time: u64,
}

impl PartialEq for StringArray {
    fn eq(&self, other: &Self) -> bool {
        self.storage.name == other.storage.name
            && self.storage.number_of_components == other.storage.number_of_components
            && self.storage.values == other.storage.values
            && self.storage.component_names == other.storage.component_names
    }
}

impl StringArray {
    /// VTK: `vtkStringArray::New`.
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
            storage: Arc::new(StringArrayStorage {
                name: name.into(),
                number_of_components,
                values: Vec::new(),
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_vec(
        name: impl Into<String>,
        values: Vec<String>,
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
            storage: Arc::new(StringArrayStorage {
                name: name.into(),
                number_of_components,
                values,
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    #[cfg(test)]
    fn from_slice(name: impl Into<String>, values: &[&str]) -> Self {
        Self::from_vec(
            name,
            values.iter().map(|value| (*value).to_string()).collect(),
            1,
        )
    }

    fn storage_mut(&mut self) -> &mut StringArrayStorage {
        Arc::make_mut(&mut self.storage)
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
        id_count_from_usize(self.storage.values.len() / self.storage.number_of_components)
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let storage = self.storage_mut();
        storage.values.resize(
            id_count_to_usize(number_of_tuples) * storage.number_of_components,
            String::new(),
        );
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> VtkIdType {
        id_count_from_usize(self.storage.values.len())
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.storage.values.is_empty()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfValues`.
    pub fn set_number_of_values(&mut self, number_of_values: VtkIdType) -> bool {
        let storage = self.storage_mut();
        storage
            .values
            .resize(id_count_to_usize(number_of_values), String::new());
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.storage.values.capacity()
    }

    /// VTK: `vtkStringArray::ReserveTuples`.
    pub fn reserve_tuples(&mut self, number_of_tuples: VtkIdType) -> bool {
        let values = id_count_to_usize(number_of_tuples) * self.storage.number_of_components;
        if values > self.storage.values.capacity() {
            let storage = self.storage_mut();
            storage.values.reserve(values - storage.values.capacity());
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
        true
    }

    /// VTK: `vtkAbstractArray::ReserveValues`.
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

    /// VTK: `vtkAbstractArray::Allocate`.
    pub fn allocate(&mut self, number_of_values: VtkIdType) -> bool {
        self.initialize();
        let storage = self.storage_mut();
        storage.values.reserve(id_count_to_usize(number_of_values));
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    /// VTK: `vtkAbstractArray::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.values.clear();
        storage.values.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::Reset`.
    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.values.clear();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkStringArray::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.values.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkStringArray::GetDataType`.
    #[cfg(test)]
    pub(crate) fn get_data_type_name(&self) -> &'static str {
        "string"
    }

    /// VTK: `vtkStringArray::GetDataTypeSize`.
    ///
    /// VTK returns `sizeof(vtkStdString)`, not the byte length of the stored
    /// string contents.
    pub fn get_data_type_size(&self) -> i32 {
        mem::size_of::<String>() as i32
    }

    /// VTK: `vtkStringArray::GetValue`.
    pub fn get_value(&self, value_idx: VtkIdType) -> &str {
        self.storage.values[id_index_to_usize(value_idx)].as_str()
    }

    /// VTK: `vtkStringArray::SetValue`.
    ///
    /// VTK expects the index to already be in range.
    pub fn set_value(&mut self, value_idx: VtkIdType, value: impl Into<String>) {
        let value_idx = id_index_to_usize(value_idx);
        assert!(
            value_idx < self.storage.values.len(),
            "value index out of range"
        );
        let storage = self.storage_mut();
        storage.values[value_idx] = value.into();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkStringArray::InsertValue`.
    ///
    /// VTK may grow the array and default-fill skipped string slots.
    pub fn insert_value(&mut self, value_idx: VtkIdType, value: impl Into<String>) {
        if value_idx >= self.get_number_of_values() {
            self.set_number_of_values(value_idx + 1);
        }
        self.set_value(value_idx, value);
    }

    /// VTK: `vtkStringArray::InsertNextValue`.
    pub fn insert_next_value(&mut self, value: impl Into<String>) -> VtkIdType {
        let value_idx = self.get_number_of_values();
        let storage = self.storage_mut();
        storage.values.push(value.into());
        storage.modified_time = storage.modified_time.saturating_add(1);
        value_idx
    }

    /// VTK: `vtkStringArray::GetTypedComponent`.
    pub fn get_typed_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> &str {
        let component_idx = component_index_to_usize(component_idx);
        assert!(
            component_idx < self.storage.number_of_components,
            "component index out of range"
        );
        self.get_value(
            id_count_from_usize(id_index_to_usize(tuple_idx) * self.storage.number_of_components)
                + component_idx as VtkIdType,
        )
    }

    /// VTK: `vtkStringArray::SetTypedComponent`.
    pub fn set_typed_component(
        &mut self,
        tuple_idx: VtkIdType,
        component_idx: i32,
        value: impl Into<String>,
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

    /// VTK: `vtkStringArray::GetTypedTuple`.
    pub fn get_typed_tuple(&self, tuple_idx: VtkIdType) -> &[String] {
        let tuple_idx = id_index_to_usize(tuple_idx);
        let start = tuple_idx * self.storage.number_of_components;
        let end = start + self.storage.number_of_components;
        &self.storage.values[start..end]
    }

    /// VTK: `vtkStringArray::SetTypedTuple`.
    pub fn set_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[String]) {
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

    pub(crate) fn insert_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[String]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        if tuple_idx >= self.get_number_of_tuples() {
            self.set_number_of_tuples(tuple_idx + 1);
        }
        self.set_typed_tuple(tuple_idx, tuple);
    }

    /// VTK: `vtkStringArray::SetTuple(i, j, source)`.
    pub fn set_tuple(&mut self, dst_tuple_idx: VtkIdType, src_tuple_idx: VtkIdType, source: &Self) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        let tuple = source.get_typed_tuple(src_tuple_idx);
        self.set_typed_tuple(dst_tuple_idx, tuple);
    }

    /// VTK: `vtkStringArray::InsertTuple(i, j, source)`.
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
        let tuple = source.get_typed_tuple(src_tuple_idx);
        self.insert_typed_tuple(dst_tuple_idx, tuple);
    }

    /// VTK: `vtkStringArray::InsertNextTuple(j, source)`.
    pub fn insert_next_tuple(&mut self, src_tuple_idx: VtkIdType, source: &Self) -> VtkIdType {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_tuple(tuple_idx, src_tuple_idx, source);
        tuple_idx
    }

    /// VTK: `vtkStringArray::GetTuples(tupleIds, output)`.
    pub fn get_tuples(&self, tuple_ids: &[VtkIdType], output: &mut Self) {
        assert_eq!(
            output.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        output.set_number_of_tuples(id_count_from_usize(tuple_ids.len()));
        for (dst_tuple_idx, &tuple_idx) in tuple_ids.iter().enumerate() {
            let tuple = self.get_typed_tuple(tuple_idx);
            output.set_typed_tuple(id_count_from_usize(dst_tuple_idx), tuple);
        }
        output.storage_mut().component_names = self.storage.component_names.clone();
    }

    /// VTK: `vtkStringArray::GetTuples(p1, p2, output)`.
    #[cfg(test)]
    pub(crate) fn get_tuples_in_range(&self, first: VtkIdType, last_inclusive: VtkIdType) -> Self {
        assert!(first <= last_inclusive, "first tuple must be <= last tuple");
        let mut output = Self::with_name_and_number_of_components(
            self.storage.name.clone(),
            self.storage.number_of_components,
        );
        let count = id_count_to_usize(last_inclusive - first + 1);
        output
            .storage_mut()
            .values
            .reserve(count * self.storage.number_of_components);
        for tuple_idx in first..=last_inclusive {
            let tuple = self.get_typed_tuple(tuple_idx);
            output.storage_mut().values.extend_from_slice(tuple);
        }
        output.storage_mut().component_names = self.storage.component_names.clone();
        output
    }

    /// VTK: `vtkStringArray::InsertTuples(dstStart, n, srcStart, source)`.
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
        let dst_start = id_index_to_usize(dst_start);
        let count = id_count_to_usize(count);
        let src_start = id_index_to_usize(src_start);
        assert!(
            src_start + count <= id_count_to_usize(source.get_number_of_tuples()),
            "source tuple index out of range"
        );
        if dst_start + count > id_count_to_usize(self.get_number_of_tuples()) {
            self.set_number_of_tuples(id_count_from_usize(dst_start + count));
        }
        let number_of_components = self.storage.number_of_components;
        let src_start_value = src_start * number_of_components;
        let src_end_value = src_start_value + count * number_of_components;
        let dst_start_value = dst_start * number_of_components;
        let dst_end_value = dst_start_value + count * number_of_components;
        let source_values = source.storage.values[src_start_value..src_end_value].to_vec();
        let storage = self.storage_mut();
        storage.values[dst_start_value..dst_end_value].clone_from_slice(&source_values);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkStringArray::InterpolateTuple(vtkIdList*, weights)`.
    pub(crate) fn interpolate_tuple_from(
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

    /// VTK: `vtkStringArray::InterpolateTuple(id1, source1, id2, source2, t)`.
    #[cfg(test)]
    pub(crate) fn interpolate_tuple_between(
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

    /// VTK: `vtkStringArray::GetDataSize`.
    ///
    /// VTK includes one terminator character per in-use string.
    pub fn get_data_size(&self) -> VtkIdType {
        id_count_from_usize(
            self.storage
                .values
                .iter()
                .map(|value| value.len() + 1)
                .sum(),
        )
    }

    /// VTK: `vtkStringArray::GetActualMemorySize`.
    ///
    /// Returned units are kibibytes. This includes the string slots reserved
    /// by the outer vector and the heap capacity held by each initialized
    /// string.
    pub fn get_actual_memory_size(&self) -> usize {
        let values = self.storage.values.capacity() * mem::size_of::<String>()
            + self
                .storage
                .values
                .iter()
                .map(String::capacity)
                .sum::<usize>();
        values.div_ceil(1024)
    }

    /// VTK: `vtkStringArray::LookupValue`.
    pub fn lookup_value(&self, value: impl ToString) -> VtkIdType {
        let value = value.to_string();
        self.lookup_typed_value(&value)
    }

    /// VTK: `vtkStringArray::LookupTypedValue`.
    pub(crate) fn lookup_typed_value(&self, value: &str) -> VtkIdType {
        self.storage
            .values
            .iter()
            .position(|candidate| candidate == value)
            .map_or(-1, id_count_from_usize)
    }

    /// VTK: `vtkStringArray::LookupValue(value, ids)`.
    pub fn lookup_value_ids(&self, value: impl ToString) -> Vec<VtkIdType> {
        let value = value.to_string();
        self.lookup_typed_value_ids(&value)
    }

    /// VTK: `vtkStringArray::LookupTypedValue(value, ids)`.
    pub(crate) fn lookup_typed_value_ids(&self, value: &str) -> Vec<VtkIdType> {
        self.storage
            .values
            .iter()
            .enumerate()
            .filter_map(|(idx, candidate)| (candidate == value).then(|| id_count_from_usize(idx)))
            .collect()
    }

    /// VTK: `vtkAbstractArray::SetComponentName`.
    pub fn set_component_name(&mut self, component: VtkIdType, name: impl Into<String>) {
        let component = id_index_to_usize(component);
        let storage = self.storage_mut();
        if component >= storage.component_names.len() {
            storage.component_names.resize_with(component + 1, || None);
        }
        storage.component_names[component] = Some(name.into());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetComponentName`.
    pub fn get_component_name(&self, component: VtkIdType) -> Option<&str> {
        self.storage
            .component_names
            .get(id_index_to_usize(component))
            .and_then(|name| name.as_deref())
    }

    /// VTK: `vtkAbstractArray::HasAComponentName`.
    pub fn has_a_component_name(&self) -> bool {
        !self.storage.component_names.is_empty()
    }

    /// VTK: `vtkAbstractArray::CopyComponentNames`.
    #[cfg(test)]
    pub(crate) fn copy_component_names_from(&mut self, other: &Self) {
        let storage = self.storage_mut();
        storage
            .component_names
            .clone_from(&other.storage.component_names);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub(crate) fn as_slice(&self) -> &[String] {
        &self.storage.values
    }

    /// VTK: `vtkStringArray::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new((*other.storage).clone());
        self.modified();
    }

    /// VTK: `vtkStringArray::ShallowCopy`.
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

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage.modified_time
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [String] {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
        &mut storage.values
    }
}

#[cfg(any())]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn insert_next_value_appends_and_returns_value_index() {
        let mut array = StringArray::with_name_and_number_of_components("labels", 1);

        assert_eq!(array.insert_next_value("left"), 0);
        assert_eq!(array.insert_next_value("right"), 1);

        assert_eq!(array.get_number_of_values(), 2);
        assert_eq!(array.get_number_of_tuples(), 2);
        assert_eq!(array.get_value(0), Some("left"));
        assert_eq!(array.get_value(1), Some("right"));
    }

    #[test]
    fn insert_value_grows_with_empty_strings() {
        let mut array = StringArray::with_name_and_number_of_components("labels", 1);

        array.insert_value(2, "third");

        assert_eq!(array.as_slice(), strings(&["", "", "third"]).as_slice());
        assert_eq!(array.get_number_of_values(), 3);
        assert_eq!(array.get_number_of_tuples(), 3);
    }

    #[test]
    fn tuple_counts_follow_components() {
        let mut array = StringArray::with_name_and_number_of_components("vectors", 2);

        array.insert_next_tuple(&strings(&["x0", "y0"]));
        array.insert_tuple(2, &strings(&["x2", "y2"]));

        assert_eq!(array.get_number_of_values(), 6);
        assert_eq!(array.get_number_of_tuples(), 3);
        assert_eq!(
            array.get_typed_tuple(0),
            Some(strings(&["x0", "y0"]).as_slice())
        );
        assert_eq!(
            array.get_typed_tuple(1),
            Some(strings(&["", ""]).as_slice())
        );
        assert_eq!(array.get_typed_component(2, 1), Some("y2"));
    }

    #[test]
    fn lookup_value_scans_flat_values_and_reports_all_matches() {
        let array = StringArray::from_vec(
            "labels",
            strings(&["left", "right", "left", "", "right", "tail"]),
            2,
        );

        assert_eq!(array.lookup_value("left"), Some(0));
        assert_eq!(array.lookup_typed_value("right"), Some(1));
        assert_eq!(array.lookup_value("tail"), Some(5));
        assert_eq!(array.lookup_value("missing"), None);
        assert_eq!(array.lookup_value_ids("right"), vec![1, 4]);
        assert_eq!(array.lookup_typed_value_ids(""), vec![3]);
        assert_eq!(array.lookup_value_ids("missing"), Vec::<usize>::new());
    }

    #[test]
    fn actual_memory_size_counts_vector_slots_and_string_capacities() {
        let mut array = StringArray::with_name_and_number_of_components("labels", 1);
        array.reserve_tuples(10);
        let mut value = String::with_capacity(1500);
        value.push_str("short");
        array.insert_next_value(value);

        let expected_bytes = array.capacity() * mem::size_of::<String>()
            + array.as_slice().iter().map(String::capacity).sum::<usize>();

        assert_eq!(
            array.get_actual_memory_size(),
            expected_bytes.div_ceil(1024)
        );
        assert_eq!(
            array.get_actual_memory_size(),
            array.get_actual_memory_size()
        );
        assert!(array.get_actual_memory_size() >= 1);
    }

    #[test]
    fn tuple_copy_helpers_copy_from_source_arrays() {
        let mut source =
            StringArray::from_vec("vectors", strings(&["x0", "y0", "x1", "y1", "x2", "y2"]), 2);
        source.set_component_name(0, "x");
        let mut dest = StringArray::with_name_and_number_of_components("dest", 2);
        dest.set_number_of_tuples(3);

        dest.set_tuple(1, 2, &source);
        assert_eq!(dest.get_typed_tuple(1), strings(&["x2", "y2"]).as_slice());

        dest.insert_tuple(4, 1, &source);
        assert_eq!(dest.get_number_of_tuples(), 5);
        assert_eq!(dest.get_typed_tuple(4), strings(&["x1", "y1"]).as_slice());

        assert_eq!(dest.insert_next_tuple(0, &source), 5);
        assert_eq!(dest.get_typed_tuple(5), strings(&["x0", "y0"]).as_slice());
    }

    #[test]
    fn get_tuples_helpers_copy_requested_tuple_order_and_names() {
        let mut array =
            StringArray::from_vec("vectors", strings(&["x0", "y0", "x1", "y1", "x2", "y2"]), 2);
        array.set_component_name(1, "y");

        let mut by_ids = StringArray::with_name_and_number_of_components("out", 2);
        array.get_tuples(&[2, 0], &mut by_ids);
        assert_eq!(by_ids.as_slice(), strings(&["x2", "y2", "x0", "y0"]));
        assert_eq!(by_ids.get_component_name(1), Some("y"));

        let by_range = array.get_tuples_in_range(1, 2);
        assert_eq!(by_range.as_slice(), strings(&["x1", "y1", "x2", "y2"]));
        assert_eq!(by_range.get_component_name(1), Some("y"));
    }

    #[test]
    fn insert_tuples_grows_and_preserves_skipped_tuples() {
        let source =
            StringArray::from_vec("vectors", strings(&["x0", "y0", "x1", "y1", "x2", "y2"]), 2);
        let mut dest = StringArray::with_name_and_number_of_components("dest", 2);

        dest.insert_tuples(1, 2, 1, &source);

        assert_eq!(dest.get_number_of_tuples(), 3);
        assert_eq!(dest.get_typed_tuple(0), Some(strings(&["", ""]).as_slice()));
        assert_eq!(
            dest.get_typed_tuple(1),
            Some(strings(&["x1", "y1"]).as_slice())
        );
        assert_eq!(
            dest.get_typed_tuple(2),
            Some(strings(&["x2", "y2"]).as_slice())
        );
    }

    #[test]
    fn initialize_and_reset_clear_different_capacity_state() {
        let mut initialized = StringArray::with_name_and_number_of_components("labels", 1);
        initialized.reserve_tuples(8);
        initialized.insert_next_value("a");
        initialized.initialize();

        assert_eq!(initialized.get_name(), "labels");
        assert_eq!(initialized.get_number_of_components(), 1);
        assert!(initialized.is_empty());
        assert_eq!(initialized.capacity(), 0);

        let mut reset = StringArray::with_name_and_number_of_components("labels", 1);
        reset.reserve_tuples(8);
        let reserved = reset.capacity();
        reset.insert_next_value("a");
        reset.reset();

        assert!(reset.is_empty());
        assert!(reset.capacity() >= reserved);
    }

    #[test]
    fn shallow_copy_shares_until_mutation() {
        let source = StringArray::from_slice("labels", &["a", "b"]);
        let mut shallow = StringArray::with_name_and_number_of_components("other", 1);

        shallow.shallow_copy(&source);

        assert!(shallow.shares_storage_with(&source));
        shallow.set_value(0, "changed");
        assert!(!shallow.shares_storage_with(&source));
        assert_eq!(source.get_value(0), Some("a"));
        assert_eq!(shallow.get_value(0), Some("changed"));
    }

    #[test]
    fn deep_copy_has_independent_storage_and_component_names() {
        let mut source = StringArray::from_slice("labels", &["a", "b"]);
        source.set_component_name(0, "name");

        let mut deep = source.deep_clone();

        assert!(!deep.shares_storage_with(&source));
        assert_eq!(deep, source);
        assert_eq!(deep.get_component_name(0), Some("name"));

        deep.set_value(0, "changed");
        assert_eq!(source.get_value(0), Some("a"));
        assert_eq!(deep.get_value(0), Some("changed"));
    }
}
