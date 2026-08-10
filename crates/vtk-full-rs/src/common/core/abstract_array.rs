use super::vtk_type::VtkDataType;
use std::{fmt, sync::Arc};

/// Numeric scalar accepted by `DataArray`.
///
/// VTK origin: numeric `vtkDataArray` subclasses whose values can be cast to
/// and from double.
pub trait Scalar: Copy + Default + Send + Sync + PartialOrd + fmt::Debug + 'static {
    const VTK_DATA_TYPE: VtkDataType;

    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! impl_scalar {
    ($ty:ty, $vtk_id:ident) => {
        impl Scalar for $ty {
            const VTK_DATA_TYPE: VtkDataType = VtkDataType::$vtk_id;

            #[inline]
            fn to_f64(self) -> f64 {
                self as f64
            }

            #[inline]
            fn from_f64(value: f64) -> Self {
                value as Self
            }
        }
    };
}

impl_scalar!(f32, Float);
impl_scalar!(f64, Double);
impl_scalar!(i8, SignedChar);
impl_scalar!(i16, Short);
impl_scalar!(i32, Int);
impl_scalar!(i64, LongLong);
impl_scalar!(u8, UnsignedChar);
impl_scalar!(u16, UnsignedShort);
impl_scalar!(u32, UnsignedInt);
impl_scalar!(u64, UnsignedLongLong);

/// Shared storage and metadata for tuple arrays.
///
/// VTK origin: audited basics from `VTK/Common/Core/vtkAbstractArray.cxx`.
#[derive(Debug, Clone)]
pub(crate) struct AbstractArray<T: Scalar> {
    storage: Arc<AbstractArrayStorage<T>>,
}

#[derive(Debug, Clone, PartialEq)]
struct AbstractArrayStorage<T: Scalar> {
    name: String,
    number_of_components: usize,
    values: Vec<T>,
    component_names: Vec<Option<String>>,
    modified_time: u64,
}

impl<T: Scalar> PartialEq for AbstractArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.storage.name == other.storage.name
            && self.storage.number_of_components == other.storage.number_of_components
            && self.storage.values == other.storage.values
            && self.storage.component_names == other.storage.component_names
    }
}

impl<T: Scalar> Default for AbstractArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Scalar> AbstractArray<T> {
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
            storage: Arc::new(AbstractArrayStorage {
                name: name.into(),
                number_of_components,
                values: Vec::new(),
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    pub(crate) fn from_vec(
        name: impl Into<String>,
        values: Vec<T>,
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
            storage: Arc::new(AbstractArrayStorage {
                name: name.into(),
                number_of_components,
                values,
                component_names: Vec::new(),
                modified_time: 0,
            }),
        }
    }

    fn storage_mut(&mut self) -> &mut AbstractArrayStorage<T> {
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
    pub fn get_number_of_components(&self) -> usize {
        self.storage.number_of_components
    }

    /// VTK: `vtkAbstractArray::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, number_of_components: usize) {
        assert!(
            number_of_components > 0,
            "number_of_components must be greater than zero"
        );
        let storage = self.storage_mut();
        storage.number_of_components = number_of_components;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> usize {
        self.storage.values.len() / self.storage.number_of_components
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: usize) {
        let storage = self.storage_mut();
        storage.values.resize(
            number_of_tuples * storage.number_of_components,
            T::default(),
        );
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> usize {
        self.storage.values.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.storage.values.capacity()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfValues`.
    pub fn set_number_of_values(&mut self, number_of_values: usize) {
        let storage = self.storage_mut();
        storage.values.resize(number_of_values, T::default());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::ReserveTuples`.
    pub fn reserve_tuples(&mut self, number_of_tuples: usize) {
        let values = number_of_tuples * self.storage.number_of_components;
        if values > self.storage.values.capacity() {
            let storage = self.storage_mut();
            storage.values.reserve(values - storage.values.capacity());
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
    }

    /// VTK: `vtkAbstractArray::ReserveValues`.
    pub fn reserve_values(&mut self, number_of_values: usize) {
        if number_of_values > self.storage.values.capacity() {
            let storage = self.storage_mut();
            storage
                .values
                .reserve(number_of_values - storage.values.capacity());
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
    }

    /// VTK: `vtkAbstractArray::Allocate`.
    pub fn allocate(&mut self, number_of_values: usize) {
        self.initialize();
        let storage = self.storage_mut();
        storage.values.reserve(number_of_values);
        storage.modified_time = storage.modified_time.saturating_add(1);
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

    /// VTK: `vtkAbstractArray::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.values.shrink_to_fit();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetDataType`.
    #[allow(dead_code)]
    pub fn get_data_type(&self) -> VtkDataType {
        T::VTK_DATA_TYPE
    }

    /// VTK: `vtkAbstractArray::GetDataTypeSize`.
    #[allow(dead_code)]
    pub fn get_data_type_size(&self) -> usize {
        T::VTK_DATA_TYPE.size()
    }

    /// VTK: `vtkAbstractArray::GetTuple`.
    pub fn get_tuple(&self, tuple_idx: usize) -> &[T] {
        let start = tuple_idx * self.storage.number_of_components;
        assert!(
            start + self.storage.number_of_components <= self.storage.values.len(),
            "tuple index out of range"
        );
        &self.storage.values[start..start + self.storage.number_of_components]
    }

    /// VTK: `vtkAbstractArray::SetTuple`.
    pub fn set_tuple(&mut self, tuple_idx: usize, tuple: &[T]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        assert!(
            tuple_idx < self.get_number_of_tuples(),
            "tuple index out of range"
        );
        let number_of_components = self.storage.number_of_components;
        let start = tuple_idx * number_of_components;
        let storage = self.storage_mut();
        storage.values[start..start + number_of_components].copy_from_slice(tuple);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::InsertTuple`.
    pub fn insert_tuple(&mut self, tuple_idx: usize, tuple: &[T]) {
        assert_eq!(
            tuple.len(),
            self.storage.number_of_components,
            "tuple component count mismatch"
        );
        if tuple_idx >= self.get_number_of_tuples() {
            self.set_number_of_tuples(tuple_idx + 1);
        }
        let number_of_components = self.storage.number_of_components;
        let start = tuple_idx * number_of_components;
        let storage = self.storage_mut();
        storage.values[start..start + number_of_components].copy_from_slice(tuple);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::InsertNextTuple`.
    pub fn insert_next_tuple(&mut self, tuple: &[T]) -> usize {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_tuple(tuple_idx, tuple);
        tuple_idx
    }

    /// VTK: `vtkGenericDataArray::RemoveTuple`.
    pub fn remove_tuple(&mut self, tuple_idx: usize) {
        if tuple_idx >= self.get_number_of_tuples() {
            return;
        }
        let number_of_components = self.storage.number_of_components;
        let start = tuple_idx * number_of_components;
        let end = start + number_of_components;
        let storage = self.storage_mut();
        storage.values.drain(start..end);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetTuples`.
    pub fn get_tuples(&self, tuple_ids: &[usize]) -> Self {
        let mut output = Self::with_name_and_number_of_components(
            self.storage.name.clone(),
            self.storage.number_of_components,
        );
        output
            .storage_mut()
            .values
            .reserve(tuple_ids.len() * self.storage.number_of_components);
        for &tuple_id in tuple_ids {
            output
                .storage_mut()
                .values
                .extend_from_slice(self.get_tuple(tuple_id));
        }
        output.storage_mut().component_names = self.storage.component_names.clone();
        output
    }

    /// VTK: `vtkAbstractArray::GetTuples(p1, p2, output)`.
    #[cfg(test)]
    pub(crate) fn get_tuples_in_range(&self, first: usize, last_inclusive: usize) -> Self {
        assert!(first <= last_inclusive, "first tuple must be <= last tuple");
        let mut output = Self::with_name_and_number_of_components(
            self.storage.name.clone(),
            self.storage.number_of_components,
        );
        let count = last_inclusive - first + 1;
        output
            .storage_mut()
            .values
            .reserve(count * self.storage.number_of_components);
        for tuple_idx in first..=last_inclusive {
            output
                .storage_mut()
                .values
                .extend_from_slice(self.get_tuple(tuple_idx));
        }
        output.storage_mut().component_names = self.storage.component_names.clone();
        output
    }

    /// VTK: `vtkAbstractArray::SetComponentName`.
    pub fn set_component_name(&mut self, component: usize, name: impl Into<String>) {
        let storage = self.storage_mut();
        if component >= storage.component_names.len() {
            storage.component_names.resize_with(component + 1, || None);
        }
        storage.component_names[component] = Some(name.into());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkAbstractArray::GetComponentName`.
    pub fn get_component_name(&self, component: usize) -> Option<&str> {
        self.storage
            .component_names
            .get(component)
            .and_then(|name| name.as_deref())
    }

    /// VTK: `vtkAbstractArray::HasAComponentName`.
    #[allow(dead_code)]
    pub fn has_a_component_name(&self) -> bool {
        !self.storage.component_names.is_empty()
    }

    /// VTK: `vtkAbstractArray::CopyComponentNames`.
    #[cfg(test)]
    pub(crate) fn copy_component_names_from(&mut self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.storage, &other.storage) || other.storage.component_names.is_empty() {
            return false;
        }
        let storage = self.storage_mut();
        storage
            .component_names
            .clone_from(&other.storage.component_names);
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    /// VTK: `vtkAbstractArray::GetVoidPointer` / `ExportToVoidPointer`.
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.storage.values
    }

    /// Mutable Rust equivalent of VTK raw data accessors.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
        &mut storage.values
    }

    /// VTK: `vtkAbstractArray::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new((*other.storage).clone());
        self.modified();
    }

    /// VTK: `vtkAbstractArray::ShallowCopy`.
    ///
    pub fn shallow_copy(&mut self, other: &Self) {
        self.storage = Arc::clone(&other.storage);
    }

    pub(crate) fn deep_clone(&self) -> Self {
        let mut output = Self::with_name_and_number_of_components(
            self.get_name(),
            self.get_number_of_components(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_array_get_tuples_by_ids_and_range() {
        let array = AbstractArray::from_vec("vectors", vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0], 2);

        assert_eq!(array.get_tuples(&[1, 0]).as_slice(), &[3.0, 4.0, 1.0, 2.0]);
        assert_eq!(
            array.get_tuples_in_range(0, 1).as_slice(),
            &[1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn abstract_array_initialize_keeps_name_and_components() {
        let mut array = AbstractArray::from_vec("scalars", vec![1i32, 2, 3], 1);

        array.initialize();

        assert_eq!(array.get_name(), "scalars");
        assert_eq!(array.get_number_of_components(), 1);
        assert!(array.as_slice().is_empty());
    }

    #[test]
    fn component_names_are_optional_and_copyable() {
        let mut array = AbstractArray::<f64>::with_name_and_number_of_components("vectors", 3);
        array.set_component_name(2, "z");

        let mut copy = AbstractArray::<f64>::with_name_and_number_of_components("copy", 3);
        assert!(copy.copy_component_names_from(&array));

        assert_eq!(copy.get_component_name(0), None);
        assert_eq!(copy.get_component_name(2), Some("z"));
        assert!(copy.has_a_component_name());
    }

    #[test]
    fn component_count_can_leave_incomplete_trailing_values() {
        let mut array = AbstractArray::from_vec("values", vec![1i32, 2, 3, 4, 5, 6], 3);

        array.set_number_of_components(4);

        assert_eq!(array.get_number_of_values(), 6);
        assert_eq!(array.get_number_of_tuples(), 1);
        assert_eq!(array.get_tuple(0), &[1, 2, 3, 4]);
    }

    #[test]
    fn shallow_copy_shares_storage_until_mutation() {
        let source = AbstractArray::from_vec("scalars", vec![1i32, 2, 3], 1);
        let mut shallow = AbstractArray::<i32>::with_name_and_number_of_components("other", 1);

        shallow.shallow_copy(&source);

        assert!(shallow.shares_storage_with(&source));
        shallow.set_tuple(0, &[9]);
        assert!(!shallow.shares_storage_with(&source));
        assert_eq!(source.get_tuple(0), &[1]);
        assert_eq!(shallow.get_tuple(0), &[9]);
    }
}
