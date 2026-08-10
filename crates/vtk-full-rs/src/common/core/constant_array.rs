use std::{marker::PhantomData, sync::Arc};

use super::{
    abstract_array::Scalar,
    data_array::{component_count_to_usize, id_count_to_usize, NativeVtkType, VtkArrayKind},
    vtk_type::{VtkDataType, VtkIdType},
};

#[derive(Debug, Clone, PartialEq)]
struct ConstantArrayStorage<T: Scalar> {
    name: String,
    value: Option<T>,
    number_of_components: usize,
    number_of_tuples: usize,
    m_time: u64,
}

/// Constant-valued implicit numeric array.
///
/// VTK origin: `VTK/Common/Core/vtkConstantArray.h` and
/// `VTK/Common/Core/vtkConstantArray.txx`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantArray<T: Scalar, K: VtkArrayKind<T> = NativeVtkType<T>> {
    storage: Arc<ConstantArrayStorage<T>>,
    kind: PhantomData<K>,
}

pub type UnsignedCharConstantArray = ConstantArray<u8>;
pub type IntConstantArray = ConstantArray<i32>;

impl<T: Scalar, K: VtkArrayKind<T>> Default for ConstantArray<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Scalar, K: VtkArrayKind<T>> ConstantArray<T, K> {
    /// VTK: `vtkConstantArray::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(ConstantArrayStorage {
                name: String::new(),
                value: None,
                number_of_components: 1,
                number_of_tuples: 0,
                m_time: 0,
            }),
            kind: PhantomData,
        }
    }

    fn storage_mut(&mut self) -> &mut ConstantArrayStorage<T> {
        Arc::make_mut(&mut self.storage)
    }

    /// VTK: `vtkConstantArray::ConstructBackend`.
    pub fn construct_backend(&mut self, value: T) {
        let storage = self.storage_mut();
        storage.value = Some(value);
        storage.m_time += 1;
    }

    /// VTK: `vtkConstantArray::GetConstantValue`.
    pub fn get_constant_value(&self) -> T {
        self.storage
            .value
            .expect("vtkConstantArray backend is null; call construct_backend first")
    }

    /// VTK: `vtkConstantArray::IsBackendConstructed`.
    pub fn is_backend_constructed(&self) -> bool {
        self.storage.value.is_some()
    }

    /// VTK: `vtkAbstractArray::GetName`.
    pub fn get_name(&self) -> &str {
        &self.storage.name
    }

    /// VTK: `vtkAbstractArray::SetName`.
    pub fn set_name(&mut self, name: impl Into<String>) {
        let storage = self.storage_mut();
        storage.name = name.into();
        storage.m_time += 1;
    }

    /// VTK: `vtkAbstractArray::GetDataType`.
    pub fn get_data_type(&self) -> VtkDataType {
        K::DATA_TYPE
    }

    /// VTK: `vtkAbstractArray::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.storage.number_of_components as i32
    }

    /// VTK: `vtkAbstractArray::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        let storage = self.storage_mut();
        storage.number_of_components = component_count_to_usize(number_of_components);
        storage.m_time += 1;
    }

    /// VTK: `vtkAbstractArray::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        self.storage.number_of_tuples as VtkIdType
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let storage = self.storage_mut();
        storage.number_of_tuples = id_count_to_usize(number_of_tuples);
        storage.m_time += 1;
    }

    /// VTK: `vtkAbstractArray::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> VtkIdType {
        (self.storage.number_of_tuples * self.storage.number_of_components) as VtkIdType
    }

    /// VTK: `vtkDataArray::GetTuple`.
    pub fn get_tuple(&self, tuple_idx: VtkIdType) -> Vec<f64> {
        self.checked_tuple(tuple_idx)
            .into_iter()
            .map(|value| value.to_f64())
            .collect()
    }

    /// VTK: `vtkDataArray::GetTuple1`.
    pub fn get_tuple1(&self, tuple_idx: VtkIdType) -> f64 {
        self.get_tuple(tuple_idx)[0]
    }

    /// VTK: `vtkGenericDataArray::GetTypedTuple`.
    pub fn get_typed_tuple(&self, tuple_idx: VtkIdType) -> Vec<T> {
        self.checked_tuple(tuple_idx)
    }

    /// VTK: `vtkGenericDataArray::GetTypedComponent`.
    pub fn get_typed_component(&self, tuple_idx: VtkIdType, component: i32) -> T {
        let _ = self.checked_tuple(tuple_idx);
        assert!(
            component >= 0 && (component as usize) < self.storage.number_of_components,
            "component index out of range"
        );
        self.get_constant_value()
    }

    fn checked_tuple(&self, tuple_idx: VtkIdType) -> Vec<T> {
        let tuple_idx = usize::try_from(tuple_idx).expect("tuple id must be non-negative");
        assert!(
            tuple_idx < self.storage.number_of_tuples,
            "tuple id out of range"
        );
        vec![self.get_constant_value(); self.storage.number_of_components]
    }

    /// VTK: `vtkAbstractArray::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.value = None;
        storage.number_of_tuples = 0;
        storage.m_time += 1;
    }

    /// VTK: `vtkAbstractArray::Reset`.
    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.number_of_tuples = 0;
        storage.m_time += 1;
    }

    /// VTK: `vtkAbstractArray::Squeeze`.
    pub fn squeeze(&mut self) {}

    /// VTK: `vtkDataArray::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        0
    }

    pub fn get_m_time(&self) -> u64 {
        self.storage.m_time
    }

    #[allow(dead_code)]
    pub(crate) fn deep_clone(&self) -> Self {
        Self {
            storage: Arc::new((*self.storage).clone()),
            kind: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            kind: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }
}
