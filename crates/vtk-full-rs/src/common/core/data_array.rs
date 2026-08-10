use super::{
    abstract_array::{AbstractArray, Scalar},
    vtk_type::{
        VtkChar, VtkDataType, VtkIdType, VtkLong, VtkTypeInt64, VtkTypeUInt64, VtkUnsignedLong,
    },
};
use std::marker::PhantomData;

pub(crate) fn component_count_to_usize(number_of_components: i32) -> usize {
    usize::try_from(number_of_components.max(1)).expect("component count must fit usize")
}

pub(crate) fn id_count_to_usize(count: VtkIdType) -> usize {
    usize::try_from(count.max(0)).expect("vtkIdType count must fit usize")
}

pub(crate) fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}

pub(crate) fn int_index_to_usize(index: i32) -> usize {
    usize::try_from(index).expect("VTK int index must be non-negative and fit usize")
}

fn vtk_double_range_sentinels() -> [f64; 2] {
    let (min, max) = VtkDataType::Double
        .range()
        .expect("VTK double range must be defined");
    [min, max]
}

fn update_range_vtk(min: &mut f64, max: &mut f64, value: f64, finite_only: bool) {
    if value.is_nan() || (finite_only && value.is_infinite()) {
        return;
    }
    if value < *min {
        *min = value;
        *max = (*max).max(value);
    } else if value > *max {
        *min = (*min).min(value);
        *max = value;
    }
}

macro_rules! define_typed_array {
    ($name:ident, $scalar:ty, $kind:ty) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            array: DataArray<$scalar, $kind>,
        }

        impl $name {
            pub fn new() -> Self {
                Self::with_name_and_number_of_components("", 1)
            }

            /// VTK: concrete typed array `ExtendedNew()`
            pub fn extended_new() -> Self {
                Self::new()
            }

            pub(crate) fn with_name_and_number_of_components(
                name: impl Into<String>,
                number_of_components: usize,
            ) -> Self {
                Self {
                    array: $crate::common::core::data_array::DataArray::with_name_and_number_of_components(
                        name,
                        number_of_components,
                    ),
                }
            }

            #[allow(dead_code)]
            pub(crate) fn from_vec(
                name: impl Into<String>,
                values: Vec<$scalar>,
                number_of_components: usize,
            ) -> Self {
                Self {
                    array: $crate::common::core::data_array::DataArray::from_vec(name, values, number_of_components),
                }
            }

            pub fn get_name(&self) -> &str {
                self.array.get_name()
            }

            pub fn set_name(&mut self, name: impl Into<String>) {
                self.array.set_name(name);
            }

            pub fn get_data_type(&self) -> VtkDataType {
                self.array.get_data_type()
            }

            pub fn get_data_type_size(&self) -> i32 {
                self.array.get_data_type_size() as i32
            }

            pub fn get_data_type_range(&self) -> [f64; 2] {
                self.array.get_data_type_range()
            }

            pub fn get_data_type_min(&self) -> f64 {
                self.array.get_data_type_min()
            }

            pub fn get_data_type_max(&self) -> f64 {
                self.array.get_data_type_max()
            }

            pub fn get_number_of_components(&self) -> i32 {
                self.array.get_number_of_components() as i32
            }

            pub fn set_number_of_components(&mut self, number_of_components: i32) {
                self.array
                    .set_number_of_components($crate::common::core::data_array::component_count_to_usize(number_of_components));
            }

            pub fn get_number_of_tuples(&self) -> VtkIdType {
                self.array.get_number_of_tuples() as VtkIdType
            }

            pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
                self.array
                    .set_number_of_tuples($crate::common::core::data_array::id_count_to_usize(number_of_tuples));
            }

            pub fn get_number_of_values(&self) -> VtkIdType {
                self.array.get_number_of_values() as VtkIdType
            }

            pub fn set_number_of_values(&mut self, number_of_values: VtkIdType) -> bool {
                self.array
                    .set_number_of_values($crate::common::core::data_array::id_count_to_usize(number_of_values));
                true
            }

            #[allow(dead_code)]
            pub(crate) fn capacity(&self) -> usize {
                self.array.capacity()
            }

            pub fn reserve_tuples(&mut self, number_of_tuples: VtkIdType) -> bool {
                self.array
                    .reserve_tuples($crate::common::core::data_array::id_count_to_usize(number_of_tuples));
                true
            }

            pub fn reserve_values(&mut self, number_of_values: VtkIdType) -> bool {
                self.array
                    .reserve_values($crate::common::core::data_array::id_count_to_usize(number_of_values));
                true
            }

            pub fn allocate(&mut self, number_of_values: VtkIdType) -> bool {
                self.array.allocate($crate::common::core::data_array::id_count_to_usize(number_of_values));
                true
            }

            #[cfg(test)]
            pub(crate) fn is_empty(&self) -> bool {
                self.array.is_empty()
            }

            pub fn initialize(&mut self) {
                self.array.initialize();
            }

            pub fn reset(&mut self) {
                self.array.reset();
            }

            pub fn squeeze(&mut self) {
                self.array.squeeze();
            }

            pub fn get_tuple(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple_n(&self, tuple_idx: VtkIdType, number_of_components: i32) -> Vec<f64> {
                self.array.get_tuple_n(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_idx),
                    usize::try_from(number_of_components.max(0)).expect("component count must fit usize"),
                )
            }

            pub fn get_tuple1(&self, tuple_idx: VtkIdType) -> f64 {
                self.array.get_tuple1($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple2(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple2($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple3(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple3($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple4(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple4($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple6(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple6($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_tuple9(&self, tuple_idx: VtkIdType) -> Vec<f64> {
                self.array.get_tuple9($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn get_integer_tuple(&self, tuple_idx: VtkIdType) -> Vec<$crate::common::core::vtk_type::VtkTypeInt64> {
                self.array.get_integer_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn set_integer_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[$crate::common::core::vtk_type::VtkTypeInt64]) {
                self.array.set_integer_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), tuple);
            }

            pub fn get_unsigned_tuple(&self, tuple_idx: VtkIdType) -> Vec<$crate::common::core::vtk_type::VtkTypeUInt64> {
                self.array.get_unsigned_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn set_unsigned_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[$crate::common::core::vtk_type::VtkTypeUInt64]) {
                self.array.set_unsigned_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), tuple);
            }

            pub fn get_typed_tuple(&self, tuple_idx: VtkIdType) -> &[$scalar] {
                self.array.get_typed_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx))
            }

            pub fn set_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[$scalar]) {
                self.array
                    .set_typed_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), tuple);
            }

            pub fn set_tuple1(&mut self, tuple_idx: VtkIdType, value: f64) {
                self.array.set_tuple1($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), value);
            }

            pub fn set_tuple2(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64) {
                self.array.set_tuple2($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1);
            }

            pub fn set_tuple3(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64) {
                self.array.set_tuple3($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2);
            }

            pub fn set_tuple4(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64) {
                self.array.set_tuple4($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3);
            }

            pub fn set_tuple6(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64) {
                self.array.set_tuple6($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3, val4, val5);
            }

            pub fn set_tuple9(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64, val6: f64, val7: f64, val8: f64) {
                self.array.set_tuple9($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3, val4, val5, val6, val7, val8);
            }

            pub fn set_tuple(
                &mut self,
                dst_tuple_idx: VtkIdType,
                src_tuple_idx: VtkIdType,
                source: &Self,
            ) {
                self.array.set_tuple(
                    $crate::common::core::data_array::vtk_id_to_usize(dst_tuple_idx),
                    $crate::common::core::data_array::vtk_id_to_usize(src_tuple_idx),
                    &source.array,
                );
            }

            pub fn insert_typed_tuple(&mut self, tuple_idx: VtkIdType, tuple: &[$scalar]) {
                self.array
                    .insert_typed_tuple($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), tuple);
            }

            pub fn insert_tuple1(&mut self, tuple_idx: VtkIdType, value: f64) {
                self.array.insert_tuple1($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), value);
            }

            pub fn insert_tuple2(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64) {
                self.array.insert_tuple2($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1);
            }

            pub fn insert_tuple3(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64) {
                self.array.insert_tuple3($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2);
            }

            pub fn insert_tuple4(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64) {
                self.array.insert_tuple4($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3);
            }

            pub fn insert_tuple6(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64) {
                self.array.insert_tuple6($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3, val4, val5);
            }

            pub fn insert_tuple9(&mut self, tuple_idx: VtkIdType, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64, val6: f64, val7: f64, val8: f64) {
                self.array.insert_tuple9($crate::common::core::data_array::vtk_id_to_usize(tuple_idx), val0, val1, val2, val3, val4, val5, val6, val7, val8);
            }

            pub(crate) fn insert_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
                self.array.insert_typed_tuple_from_f64(tuple_idx, tuple);
            }

            pub(crate) fn set_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
                self.array.set_typed_tuple_from_f64(tuple_idx, tuple);
            }

            pub fn insert_tuple(
                &mut self,
                dst_tuple_idx: VtkIdType,
                src_tuple_idx: VtkIdType,
                source: &Self,
            ) {
                self.array.insert_tuple(
                    $crate::common::core::data_array::vtk_id_to_usize(dst_tuple_idx),
                    $crate::common::core::data_array::vtk_id_to_usize(src_tuple_idx),
                    &source.array,
                );
            }

            pub fn insert_next_typed_tuple(&mut self, tuple: &[$scalar]) -> VtkIdType {
                self.array.insert_next_typed_tuple(tuple) as VtkIdType
            }

            pub fn insert_next_tuple1(&mut self, value: f64) {
                self.array.insert_next_tuple1(value);
            }

            pub fn insert_next_tuple2(&mut self, val0: f64, val1: f64) {
                self.array.insert_next_tuple2(val0, val1);
            }

            pub fn insert_next_tuple3(&mut self, val0: f64, val1: f64, val2: f64) {
                self.array.insert_next_tuple3(val0, val1, val2);
            }

            pub fn insert_next_tuple4(&mut self, val0: f64, val1: f64, val2: f64, val3: f64) {
                self.array.insert_next_tuple4(val0, val1, val2, val3);
            }

            pub fn insert_next_tuple6(&mut self, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64) {
                self.array.insert_next_tuple6(val0, val1, val2, val3, val4, val5);
            }

            pub fn insert_next_tuple9(&mut self, val0: f64, val1: f64, val2: f64, val3: f64, val4: f64, val5: f64, val6: f64, val7: f64, val8: f64) {
                self.array.insert_next_tuple9(val0, val1, val2, val3, val4, val5, val6, val7, val8);
            }

            pub fn insert_next_tuple(
                &mut self,
                src_tuple_idx: VtkIdType,
                source: &Self,
            ) -> VtkIdType {
                self.array
                    .insert_next_tuple(vtk_id_to_usize(src_tuple_idx), &source.array)
                    as VtkIdType
            }

            pub fn remove_tuple(&mut self, tuple_idx: VtkIdType) {
                if tuple_idx < 0 {
                    return;
                }
                self.array.remove_tuple(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_idx),
                );
            }

            pub fn remove_last_tuple(&mut self) {
                self.array.remove_last_tuple();
            }

            pub fn get_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> f64 {
                self.array.get_component(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_idx),
                    $crate::common::core::data_array::int_index_to_usize(component_idx),
                )
            }

            pub fn set_component(&mut self, tuple_idx: VtkIdType, component_idx: i32, value: f64) {
                self.array.set_component(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_idx),
                    $crate::common::core::data_array::int_index_to_usize(component_idx),
                    value,
                );
            }

            pub fn insert_component(
                &mut self,
                tuple_idx: VtkIdType,
                component_idx: i32,
                value: f64,
            ) {
                self.array.insert_component(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_idx),
                    $crate::common::core::data_array::int_index_to_usize(component_idx),
                    value,
                );
            }

            pub fn fill_component(&mut self, component_idx: i32, value: f64) {
                self.array
                    .fill_component($crate::common::core::data_array::int_index_to_usize(component_idx), value);
            }

            pub fn fill(&mut self, value: f64) {
                self.array.fill(value);
            }

            pub fn get_tuples(&self, tuple_ids: &[VtkIdType], output: &mut Self) {
                let tuple_ids: Vec<_> = tuple_ids
                    .iter()
                    .map(|&id| $crate::common::core::data_array::vtk_id_to_usize(id))
                    .collect();
                self.array.get_tuples(&tuple_ids, &mut output.array);
            }

            pub fn get_tuples_in_range(
                &self,
                first: VtkIdType,
                last_inclusive: VtkIdType,
                output: &mut Self,
            ) {
                self.array.get_tuples_in_range(
                    $crate::common::core::data_array::vtk_id_to_usize(first),
                    $crate::common::core::data_array::vtk_id_to_usize(last_inclusive),
                    &mut output.array,
                );
            }

            pub fn insert_tuples(
                &mut self,
                dst_start: VtkIdType,
                count: VtkIdType,
                src_start: VtkIdType,
                source: &Self,
            ) {
                self.array.insert_tuples(
                    $crate::common::core::data_array::vtk_id_to_usize(dst_start),
                    $crate::common::core::data_array::id_count_to_usize(count),
                    $crate::common::core::data_array::vtk_id_to_usize(src_start),
                    &source.array,
                );
            }

            pub fn insert_tuples_by_ids(
                &mut self,
                dst_ids: &[VtkIdType],
                src_ids: &[VtkIdType],
                source: &Self,
            ) {
                let dst_ids: Vec<_> = dst_ids
                    .iter()
                    .map(|&id| $crate::common::core::data_array::vtk_id_to_usize(id))
                    .collect();
                let src_ids: Vec<_> = src_ids
                    .iter()
                    .map(|&id| $crate::common::core::data_array::vtk_id_to_usize(id))
                    .collect();
                self.array
                    .insert_tuples_by_ids(&dst_ids, &src_ids, &source.array);
            }

            pub fn insert_tuples_starting_at(
                &mut self,
                dst_start: VtkIdType,
                src_ids: &[VtkIdType],
                source: &Self,
            ) {
                let src_ids: Vec<_> = src_ids
                    .iter()
                    .map(|&id| $crate::common::core::data_array::vtk_id_to_usize(id))
                    .collect();
                self.array.insert_tuples_starting_at(
                    $crate::common::core::data_array::vtk_id_to_usize(dst_start),
                    &src_ids,
                    &source.array,
                );
            }

            pub fn set_component_name(&mut self, component: VtkIdType, name: impl Into<String>) {
                self.array
                    .set_component_name($crate::common::core::data_array::vtk_id_to_usize(component), name);
            }

            pub fn get_component_name(&self, component: VtkIdType) -> Option<&str> {
                self.array.get_component_name($crate::common::core::data_array::vtk_id_to_usize(component))
            }

            pub(crate) fn has_a_component_name(&self) -> bool {
                self.array.has_a_component_name()
            }

            #[cfg(test)]
            pub(crate) fn copy_component_names_from(&mut self, other: &Self) -> bool {
                self.array.copy_component_names_from(&other.array)
            }

            pub fn get_data(
                &self,
                tuple_min: VtkIdType,
                tuple_max: VtkIdType,
                component_min: i32,
                component_max: i32,
            ) -> Vec<f64> {
                self.array.get_data(
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_min),
                    $crate::common::core::data_array::vtk_id_to_usize(tuple_max),
                    $crate::common::core::data_array::int_index_to_usize(component_min),
                    $crate::common::core::data_array::int_index_to_usize(component_max),
                )
            }

            pub(crate) fn tuple_as_f64(&self, tuple_idx: usize) -> Vec<f64> {
                self.array.tuple_as_f64(tuple_idx)
            }

            pub(crate) fn checked_tuple_as_f64(
                &self,
                tuple_idx: usize,
            ) -> Result<Vec<f64>, crate::common::core::ArrayError> {
                let number_of_tuples = self.array.get_number_of_tuples();
                if tuple_idx >= number_of_tuples {
                    return Err(crate::common::core::ArrayError::TupleOutOfRange {
                        tuple: tuple_idx,
                        number_of_tuples,
                    });
                }
                Ok(self.tuple_as_f64(tuple_idx))
            }

            pub fn get_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
                self.array.get_range_with_component(component)
            }

            pub fn get_finite_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
                self.array.get_finite_range_with_component(component)
            }

            pub fn get_range(&self) -> Option<[f64; 2]> {
                self.array.get_range()
            }

            pub fn get_max_norm(&self) -> f64 {
                self.array.get_max_norm()
            }

            pub fn get_actual_memory_size(&self) -> usize {
                self.array.get_actual_memory_size()
            }

            #[allow(dead_code)]
            pub(crate) fn as_slice(&self) -> &[$scalar] {
                self.array.as_slice()
            }

            #[allow(dead_code)]
            pub(crate) fn as_mut_slice(&mut self) -> &mut [$scalar] {
                self.array.as_mut_slice()
            }

            #[allow(dead_code)]
            pub(crate) fn iter_tuples(&self) -> $crate::common::core::data_array::DataArrayTupleIter<'_, $scalar, $kind> {
                self.array.iter_tuples()
            }

            pub fn deep_copy(&mut self, other: &Self) {
                self.array.deep_copy(&other.array);
            }

            pub fn shallow_copy(&mut self, other: &Self) {
                self.array.shallow_copy(&other.array);
            }

            pub(crate) fn deep_clone(&self) -> Self {
                Self {
                    array: self.array.deep_clone(),
                }
            }

            pub(crate) fn shallow_clone(&self) -> Self {
                Self {
                    array: self.array.shallow_clone(),
                }
            }

            pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
                self.array.shares_storage_with(&other.array)
            }

            pub fn get_m_time(&self) -> u64 {
                self.array.get_m_time()
            }

            pub fn compute_scalar_range(&self, ranges: &mut [f64]) -> bool {
                self.array.compute_scalar_range(ranges)
            }

            pub fn compute_scalar_range_with_ghosts(
                &self,
                ranges: &mut [f64],
                ghosts: Option<&[u8]>,
                ghosts_to_skip: u8,
            ) -> bool {
                self.array
                    .compute_scalar_range_with_ghosts(ranges, ghosts, ghosts_to_skip)
            }

            pub fn compute_finite_scalar_range(&self, ranges: &mut [f64]) -> bool {
                self.array.compute_finite_scalar_range(ranges)
            }

            pub fn compute_finite_scalar_range_with_ghosts(
                &self,
                ranges: &mut [f64],
                ghosts: Option<&[u8]>,
                ghosts_to_skip: u8,
            ) -> bool {
                self.array
                    .compute_finite_scalar_range_with_ghosts(ranges, ghosts, ghosts_to_skip)
            }

            pub fn compute_vector_range(&self, range: &mut [f64]) -> bool {
                self.array.compute_vector_range(range)
            }

            pub fn compute_vector_range_with_ghosts(
                &self,
                range: &mut [f64],
                ghosts: Option<&[u8]>,
                ghosts_to_skip: u8,
            ) -> bool {
                self.array
                    .compute_vector_range_with_ghosts(range, ghosts, ghosts_to_skip)
            }

            pub fn compute_finite_vector_range(&self, range: &mut [f64]) -> bool {
                self.array.compute_finite_vector_range(range)
            }

            pub fn compute_finite_vector_range_with_ghosts(
                &self,
                range: &mut [f64],
                ghosts: Option<&[u8]>,
                ghosts_to_skip: u8,
            ) -> bool {
                self.array
                    .compute_finite_vector_range_with_ghosts(range, ghosts, ghosts_to_skip)
            }
        }
    };
}

pub(crate) use define_typed_array;

pub trait VtkArrayKind<T: Scalar>:
    Clone + Copy + Default + PartialEq + std::fmt::Debug + 'static
{
    const DATA_TYPE: VtkDataType;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeVtkType<T: Scalar>(PhantomData<T>);

impl<T: Scalar> VtkArrayKind<T> for NativeVtkType<T> {
    const DATA_TYPE: VtkDataType = T::VTK_DATA_TYPE;
}

macro_rules! define_array_kind {
    ($kind:ident, $scalar:ty, $data_type:ident) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub(crate) struct $kind;

        impl VtkArrayKind<$scalar> for $kind {
            const DATA_TYPE: VtkDataType = VtkDataType::$data_type;
        }
    };
}

define_array_kind!(CharKind, VtkChar, Char);
define_array_kind!(LongKind, VtkLong, Long);
define_array_kind!(UnsignedLongKind, VtkUnsignedLong, UnsignedLong);
define_array_kind!(IdTypeKind, VtkIdType, IdType);

/// Numeric tuple array with VTK-shaped component access.
///
/// VTK origin: audited basics from `VTK/Common/Core/vtkDataArray.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataArray<T: Scalar, K: VtkArrayKind<T> = NativeVtkType<T>> {
    storage: AbstractArray<T>,
    kind: PhantomData<K>,
}

impl<T: Scalar, K: VtkArrayKind<T>> Default for DataArray<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Scalar, K: VtkArrayKind<T>> DataArray<T, K> {
    pub fn new() -> Self {
        Self::with_name_and_number_of_components("", 1)
    }

    pub(crate) fn with_name_and_number_of_components(
        name: impl Into<String>,
        number_of_components: usize,
    ) -> Self {
        Self {
            storage: AbstractArray::with_name_and_number_of_components(name, number_of_components),
            kind: PhantomData,
        }
    }

    pub(crate) fn from_vec(
        name: impl Into<String>,
        values: Vec<T>,
        number_of_components: usize,
    ) -> Self {
        Self {
            storage: AbstractArray::from_vec(name, values, number_of_components),
            kind: PhantomData,
        }
    }

    /// VTK: `vtkAbstractArray::GetName`.
    pub fn get_name(&self) -> &str {
        self.storage.get_name()
    }

    /// VTK: `vtkAbstractArray::SetName`.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.storage.set_name(name);
    }

    /// VTK: `vtkAbstractArray::GetDataType`.
    pub fn get_data_type(&self) -> VtkDataType {
        K::DATA_TYPE
    }

    /// VTK: `vtkDataArray::GetDataTypeSize`.
    pub fn get_data_type_size(&self) -> usize {
        K::DATA_TYPE.size()
    }

    /// VTK: `vtkDataArray::GetDataTypeRange`.
    pub fn get_data_type_range(&self) -> [f64; 2] {
        let (min, max) = K::DATA_TYPE
            .range()
            .expect("numeric DataArray kind must have a VTK numeric range");
        [min, max]
    }

    /// VTK: `vtkDataArray::GetDataTypeMin`.
    pub fn get_data_type_min(&self) -> f64 {
        self.get_data_type_range()[0]
    }

    /// VTK: `vtkDataArray::GetDataTypeMax`.
    pub fn get_data_type_max(&self) -> f64 {
        self.get_data_type_range()[1]
    }

    /// VTK: `vtkAbstractArray::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> usize {
        self.storage.get_number_of_components()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, number_of_components: usize) {
        self.storage.set_number_of_components(number_of_components);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> usize {
        self.storage.get_number_of_tuples()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfTuples`.
    pub fn set_number_of_tuples(&mut self, number_of_tuples: usize) {
        self.storage.set_number_of_tuples(number_of_tuples);
    }

    /// VTK: `vtkAbstractArray::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> usize {
        self.storage.get_number_of_values()
    }

    /// VTK: `vtkAbstractArray::SetNumberOfValues`.
    pub fn set_number_of_values(&mut self, number_of_values: usize) {
        self.storage.set_number_of_values(number_of_values);
    }

    pub(crate) fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// VTK: `vtkAbstractArray::ReserveTuples`.
    pub fn reserve_tuples(&mut self, number_of_tuples: usize) {
        self.storage.reserve_tuples(number_of_tuples);
    }

    /// VTK: `vtkAbstractArray::ReserveValues`.
    pub fn reserve_values(&mut self, number_of_values: usize) {
        self.storage.reserve_values(number_of_values);
    }

    /// VTK: `vtkAbstractArray::Allocate`.
    pub fn allocate(&mut self, number_of_values: usize) {
        self.storage.allocate(number_of_values);
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.get_number_of_values() == 0
    }

    /// VTK: `vtkAbstractArray::Initialize`.
    pub fn initialize(&mut self) {
        self.storage.initialize();
    }

    /// VTK: `vtkAbstractArray::Reset`.
    pub fn reset(&mut self) {
        self.storage.reset();
    }

    /// VTK: `vtkAbstractArray::Squeeze`.
    pub fn squeeze(&mut self) {
        self.storage.squeeze();
    }

    /// VTK: `vtkDataArray::GetTuple`.
    pub fn get_tuple(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_typed_tuple(tuple_idx)
            .iter()
            .map(|value| value.to_f64())
            .collect()
    }

    /// VTK: `vtkDataArray::GetTupleN`.
    pub fn get_tuple_n(&self, tuple_idx: usize, _number_of_components: usize) -> Vec<f64> {
        self.get_tuple(tuple_idx)
    }

    /// VTK: `vtkDataArray::GetTuple1`.
    pub fn get_tuple1(&self, tuple_idx: usize) -> f64 {
        self.get_tuple(tuple_idx)[0]
    }

    /// VTK: `vtkDataArray::GetTuple2`.
    pub fn get_tuple2(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple_n(tuple_idx, 2)
    }

    /// VTK: `vtkDataArray::GetTuple3`.
    pub fn get_tuple3(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple_n(tuple_idx, 3)
    }

    /// VTK: `vtkDataArray::GetTuple4`.
    pub fn get_tuple4(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple_n(tuple_idx, 4)
    }

    /// VTK: `vtkDataArray::GetTuple6`.
    pub fn get_tuple6(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple_n(tuple_idx, 6)
    }

    /// VTK: `vtkDataArray::GetTuple9`.
    pub fn get_tuple9(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple_n(tuple_idx, 9)
    }

    /// VTK: `vtkDataArray::GetIntegerTuple`.
    pub fn get_integer_tuple(&self, tuple_idx: usize) -> Vec<VtkTypeInt64> {
        self.get_tuple(tuple_idx)
            .into_iter()
            .map(|value| value as VtkTypeInt64)
            .collect()
    }

    /// VTK: `vtkDataArray::SetIntegerTuple`.
    pub fn set_integer_tuple(&mut self, tuple_idx: usize, tuple: &[VtkTypeInt64]) {
        let tuple: Vec<_> = tuple.iter().map(|&value| value as f64).collect();
        self.set_typed_tuple_from_f64(tuple_idx, &tuple);
    }

    /// VTK: `vtkDataArray::GetUnsignedTuple`.
    pub fn get_unsigned_tuple(&self, tuple_idx: usize) -> Vec<VtkTypeUInt64> {
        self.get_tuple(tuple_idx)
            .into_iter()
            .map(|value| value as VtkTypeUInt64)
            .collect()
    }

    /// VTK: `vtkDataArray::SetUnsignedTuple`.
    pub fn set_unsigned_tuple(&mut self, tuple_idx: usize, tuple: &[VtkTypeUInt64]) {
        let tuple: Vec<_> = tuple.iter().map(|&value| value as f64).collect();
        self.set_typed_tuple_from_f64(tuple_idx, &tuple);
    }

    /// VTK: `vtkGenericDataArray::GetTypedTuple`.
    pub fn get_typed_tuple(&self, tuple_idx: usize) -> &[T] {
        self.storage.get_tuple(tuple_idx)
    }

    /// VTK: `vtkGenericDataArray::SetTypedTuple`.
    pub fn set_typed_tuple(&mut self, tuple_idx: usize, tuple: &[T]) {
        self.storage.set_tuple(tuple_idx, tuple);
    }

    /// VTK: `vtkDataArray::SetTuple1`.
    pub fn set_tuple1(&mut self, tuple_idx: usize, value: f64) {
        self.set_tuple_fixed(tuple_idx, 1, &[value]);
    }

    /// VTK: `vtkDataArray::SetTuple2`.
    pub fn set_tuple2(&mut self, tuple_idx: usize, val0: f64, val1: f64) {
        self.set_tuple_fixed(tuple_idx, 2, &[val0, val1]);
    }

    /// VTK: `vtkDataArray::SetTuple3`.
    pub fn set_tuple3(&mut self, tuple_idx: usize, val0: f64, val1: f64, val2: f64) {
        self.set_tuple_fixed(tuple_idx, 3, &[val0, val1, val2]);
    }

    /// VTK: `vtkDataArray::SetTuple4`.
    pub fn set_tuple4(&mut self, tuple_idx: usize, val0: f64, val1: f64, val2: f64, val3: f64) {
        self.set_tuple_fixed(tuple_idx, 4, &[val0, val1, val2, val3]);
    }

    /// VTK: `vtkDataArray::SetTuple6`.
    pub fn set_tuple6(
        &mut self,
        tuple_idx: usize,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
    ) {
        self.set_tuple_fixed(tuple_idx, 6, &[val0, val1, val2, val3, val4, val5]);
    }

    /// VTK: `vtkDataArray::SetTuple9`.
    pub fn set_tuple9(
        &mut self,
        tuple_idx: usize,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
        val6: f64,
        val7: f64,
        val8: f64,
    ) {
        self.set_tuple_fixed(
            tuple_idx,
            9,
            &[val0, val1, val2, val3, val4, val5, val6, val7, val8],
        );
    }

    /// VTK: `vtkDataArray::SetTuple(tupleIdx, const double*)`.
    pub(crate) fn set_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
        assert_eq!(
            tuple.len(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        let converted: Vec<T> = tuple.iter().map(|&value| T::from_f64(value)).collect();
        self.set_typed_tuple(tuple_idx, &converted);
    }

    /// VTK: `vtkGenericDataArray::InsertTypedTuple`.
    pub fn insert_typed_tuple(&mut self, tuple_idx: usize, tuple: &[T]) {
        self.storage.insert_tuple(tuple_idx, tuple);
    }

    /// VTK: `vtkDataArray::InsertTuple1`.
    pub fn insert_tuple1(&mut self, tuple_idx: usize, value: f64) {
        self.insert_tuple_fixed(tuple_idx, 1, &[value]);
    }

    /// VTK: `vtkDataArray::InsertTuple2`.
    pub fn insert_tuple2(&mut self, tuple_idx: usize, val0: f64, val1: f64) {
        self.insert_tuple_fixed(tuple_idx, 2, &[val0, val1]);
    }

    /// VTK: `vtkDataArray::InsertTuple3`.
    pub fn insert_tuple3(&mut self, tuple_idx: usize, val0: f64, val1: f64, val2: f64) {
        self.insert_tuple_fixed(tuple_idx, 3, &[val0, val1, val2]);
    }

    /// VTK: `vtkDataArray::InsertTuple4`.
    pub fn insert_tuple4(&mut self, tuple_idx: usize, val0: f64, val1: f64, val2: f64, val3: f64) {
        self.insert_tuple_fixed(tuple_idx, 4, &[val0, val1, val2, val3]);
    }

    /// VTK: `vtkDataArray::InsertTuple6`.
    pub fn insert_tuple6(
        &mut self,
        tuple_idx: usize,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
    ) {
        self.insert_tuple_fixed(tuple_idx, 6, &[val0, val1, val2, val3, val4, val5]);
    }

    /// VTK: `vtkDataArray::InsertTuple9`.
    pub fn insert_tuple9(
        &mut self,
        tuple_idx: usize,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
        val6: f64,
        val7: f64,
        val8: f64,
    ) {
        self.insert_tuple_fixed(
            tuple_idx,
            9,
            &[val0, val1, val2, val3, val4, val5, val6, val7, val8],
        );
    }

    /// VTK: `vtkDataArray::InsertTuple(tupleIdx, const double*)`.
    pub(crate) fn insert_typed_tuple_from_f64(&mut self, tuple_idx: usize, tuple: &[f64]) {
        assert_eq!(
            tuple.len(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        let converted: Vec<T> = tuple.iter().map(|&value| T::from_f64(value)).collect();
        self.insert_typed_tuple(tuple_idx, &converted);
    }

    /// VTK: `vtkGenericDataArray::InsertNextTypedTuple`.
    pub fn insert_next_typed_tuple(&mut self, tuple: &[T]) -> usize {
        self.storage.insert_next_tuple(tuple)
    }

    /// VTK: `vtkDataArray::InsertNextTuple1`.
    pub fn insert_next_tuple1(&mut self, value: f64) {
        self.insert_next_tuple_fixed(1, &[value]);
    }

    /// VTK: `vtkDataArray::InsertNextTuple2`.
    pub fn insert_next_tuple2(&mut self, val0: f64, val1: f64) {
        self.insert_next_tuple_fixed(2, &[val0, val1]);
    }

    /// VTK: `vtkDataArray::InsertNextTuple3`.
    pub fn insert_next_tuple3(&mut self, val0: f64, val1: f64, val2: f64) {
        self.insert_next_tuple_fixed(3, &[val0, val1, val2]);
    }

    /// VTK: `vtkDataArray::InsertNextTuple4`.
    pub fn insert_next_tuple4(&mut self, val0: f64, val1: f64, val2: f64, val3: f64) {
        self.insert_next_tuple_fixed(4, &[val0, val1, val2, val3]);
    }

    /// VTK: `vtkDataArray::InsertNextTuple6`.
    pub fn insert_next_tuple6(
        &mut self,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
    ) {
        self.insert_next_tuple_fixed(6, &[val0, val1, val2, val3, val4, val5]);
    }

    /// VTK: `vtkDataArray::InsertNextTuple9`.
    pub fn insert_next_tuple9(
        &mut self,
        val0: f64,
        val1: f64,
        val2: f64,
        val3: f64,
        val4: f64,
        val5: f64,
        val6: f64,
        val7: f64,
        val8: f64,
    ) {
        self.insert_next_tuple_fixed(9, &[val0, val1, val2, val3, val4, val5, val6, val7, val8]);
    }

    fn set_tuple_fixed(&mut self, tuple_idx: usize, expected_components: usize, tuple: &[f64]) {
        if self.get_number_of_components() != expected_components {
            return;
        }
        self.set_typed_tuple_from_f64(tuple_idx, tuple);
    }

    fn insert_tuple_fixed(&mut self, tuple_idx: usize, expected_components: usize, tuple: &[f64]) {
        if self.get_number_of_components() != expected_components {
            return;
        }
        self.insert_typed_tuple_from_f64(tuple_idx, tuple);
    }

    fn insert_next_tuple_fixed(&mut self, expected_components: usize, tuple: &[f64]) {
        if self.get_number_of_components() != expected_components {
            return;
        }
        self.insert_typed_tuple_from_f64(self.get_number_of_tuples(), tuple);
    }

    /// VTK: `vtkGenericDataArray::RemoveTuple`.
    pub fn remove_tuple(&mut self, tuple_idx: usize) {
        self.storage.remove_tuple(tuple_idx);
    }

    /// VTK: `vtkDataArray::RemoveLastTuple`.
    pub fn remove_last_tuple(&mut self) {
        if self.get_number_of_tuples() > 0 {
            self.set_number_of_tuples(self.get_number_of_tuples() - 1);
            self.squeeze();
        }
    }

    /// VTK: `vtkDataArray::GetComponent`.
    pub fn get_component(&self, tuple_idx: usize, component_idx: usize) -> f64 {
        assert!(
            component_idx < self.get_number_of_components(),
            "component index out of range"
        );
        let value_idx = tuple_idx * self.get_number_of_components() + component_idx;
        assert!(
            value_idx < self.get_number_of_values(),
            "component index out of range"
        );
        self.storage.as_slice()[value_idx].to_f64()
    }

    /// VTK: `vtkDataArray::SetComponent`.
    pub fn set_component(&mut self, tuple_idx: usize, component_idx: usize, value: f64) {
        assert!(
            component_idx < self.get_number_of_components(),
            "component index out of range"
        );
        let mut tuple = if tuple_idx < self.get_number_of_tuples() {
            self.get_typed_tuple(tuple_idx).to_vec()
        } else {
            vec![T::default(); self.get_number_of_components()]
        };
        tuple[component_idx] = T::from_f64(value);
        self.storage.insert_tuple(tuple_idx, &tuple);
    }

    /// VTK: `vtkDataArray::InsertComponent`.
    pub fn insert_component(&mut self, tuple_idx: usize, component_idx: usize, value: f64) {
        assert!(
            component_idx < self.get_number_of_components(),
            "component index out of range"
        );
        let mut tuple = if tuple_idx < self.get_number_of_tuples() {
            self.get_typed_tuple(tuple_idx).to_vec()
        } else {
            vec![T::default(); self.get_number_of_components()]
        };
        tuple[component_idx] = T::from_f64(value);
        self.storage.insert_tuple(tuple_idx, &tuple);
    }

    /// VTK: `vtkDataArray::FillComponent`.
    pub fn fill_component(&mut self, component_idx: usize, value: f64) {
        assert!(
            component_idx < self.get_number_of_components(),
            "component index out of range"
        );
        let value = T::from_f64(value);
        let number_of_components = self.get_number_of_components();
        for tuple_idx in 0..self.get_number_of_tuples() {
            self.storage.as_mut_slice()[tuple_idx * number_of_components + component_idx] = value;
        }
        self.storage.modified();
    }

    /// VTK: `vtkDataArray::Fill`.
    pub fn fill(&mut self, value: f64) {
        let value = T::from_f64(value);
        for item in self.storage.as_mut_slice() {
            *item = value;
        }
        self.storage.modified();
    }

    /// VTK: `vtkDataArray::GetTuples(tupleIds, output)`.
    pub fn get_tuples(&self, tuple_ids: &[usize], output: &mut Self) {
        assert_eq!(
            output.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        assert!(
            output.get_number_of_tuples() >= tuple_ids.len(),
            "output array must have enough tuples"
        );
        for (dst_tuple_idx, &src_tuple_idx) in tuple_ids.iter().enumerate() {
            output.set_tuple(dst_tuple_idx, src_tuple_idx, self);
        }
    }

    /// VTK: `vtkDataArray::GetTuples(p1, p2, output)`.
    pub fn get_tuples_in_range(&self, first: usize, last_inclusive: usize, output: &mut Self) {
        assert_eq!(
            output.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        assert!(first <= last_inclusive, "first tuple must be <= last tuple");
        let count = last_inclusive - first + 1;
        assert!(
            output.get_number_of_tuples() >= count,
            "output array must have enough tuples"
        );
        for (dst_tuple_idx, src_tuple_idx) in (first..=last_inclusive).enumerate() {
            output.set_tuple(dst_tuple_idx, src_tuple_idx, self);
        }
    }

    /// VTK: `vtkDataArray::InsertTuples(dstStart, n, srcStart, source)`.
    pub fn insert_tuples(
        &mut self,
        dst_start: usize,
        count: usize,
        src_start: usize,
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
        if dst_start + count > self.get_number_of_tuples() {
            self.set_number_of_tuples(dst_start + count);
        }
        for offset in 0..count {
            self.set_tuple(dst_start + offset, src_start + offset, source);
        }
    }

    /// VTK: `vtkDataArray::InsertTuples(dstIds, srcIds, source)`.
    pub fn insert_tuples_by_ids(&mut self, dst_ids: &[usize], src_ids: &[usize], source: &Self) {
        if dst_ids.is_empty() {
            return;
        }
        assert_eq!(
            dst_ids.len(),
            src_ids.len(),
            "source and destination id counts must match"
        );
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );

        let max_src_tuple_id = *src_ids.iter().max().expect("non-empty source ids");
        let max_dst_tuple_id = *dst_ids.iter().max().expect("non-empty destination ids");
        assert!(
            max_src_tuple_id < source.get_number_of_tuples(),
            "source tuple index out of range"
        );
        if max_dst_tuple_id >= self.get_number_of_tuples() {
            self.set_number_of_tuples(max_dst_tuple_id + 1);
        }
        for (&dst_tuple_idx, &src_tuple_idx) in dst_ids.iter().zip(src_ids) {
            self.set_tuple(dst_tuple_idx, src_tuple_idx, source);
        }
    }

    /// VTK: `vtkDataArray::InsertTuplesStartingAt(dstStart, srcIds, source)`.
    pub fn insert_tuples_starting_at(
        &mut self,
        dst_start: usize,
        src_ids: &[usize],
        source: &Self,
    ) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );

        let max_src_tuple_id = *src_ids.iter().max().expect("source ids must be non-empty");
        assert!(
            max_src_tuple_id < source.get_number_of_tuples(),
            "source tuple index out of range"
        );
        let count = src_ids.len();
        if dst_start + count > self.get_number_of_tuples() {
            self.set_number_of_tuples(dst_start + count);
        }
        for (offset, &src_tuple_idx) in src_ids.iter().enumerate() {
            self.set_tuple(dst_start + offset, src_tuple_idx, source);
        }
    }

    /// VTK: `vtkAbstractArray::SetTuple(i, j, source)`.
    pub fn set_tuple(&mut self, dst_tuple_idx: usize, src_tuple_idx: usize, source: &Self) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        self.set_typed_tuple(dst_tuple_idx, source.get_typed_tuple(src_tuple_idx));
    }

    /// VTK: `vtkAbstractArray::InsertTuple(i, j, source)`.
    pub fn insert_tuple(&mut self, dst_tuple_idx: usize, src_tuple_idx: usize, source: &Self) {
        assert_eq!(
            source.get_number_of_components(),
            self.get_number_of_components(),
            "tuple component count mismatch"
        );
        self.insert_typed_tuple(dst_tuple_idx, source.get_typed_tuple(src_tuple_idx));
    }

    /// VTK: `vtkAbstractArray::InsertNextTuple(j, source)`.
    pub fn insert_next_tuple(&mut self, src_tuple_idx: usize, source: &Self) -> usize {
        let tuple_idx = self.get_number_of_tuples();
        self.insert_tuple(tuple_idx, src_tuple_idx, source);
        tuple_idx
    }

    /// VTK: `vtkAbstractArray::SetComponentName`.
    pub fn set_component_name(&mut self, component: usize, name: impl Into<String>) {
        self.storage.set_component_name(component, name);
    }

    /// VTK: `vtkAbstractArray::GetComponentName`.
    pub fn get_component_name(&self, component: usize) -> Option<&str> {
        self.storage.get_component_name(component)
    }

    /// VTK: `vtkAbstractArray::HasAComponentName`.
    pub(crate) fn has_a_component_name(&self) -> bool {
        self.storage.has_a_component_name()
    }

    /// VTK: `vtkAbstractArray::CopyComponentNames`.
    #[cfg(test)]
    pub(crate) fn copy_component_names_from(&mut self, other: &Self) -> bool {
        self.storage.copy_component_names_from(&other.storage)
    }

    /// VTK: `vtkDataArray::GetData`.
    pub fn get_data(
        &self,
        tuple_min: usize,
        tuple_max: usize,
        component_min: usize,
        component_max: usize,
    ) -> Vec<f64> {
        assert!(tuple_min <= tuple_max, "tuple_min must be <= tuple_max");
        assert!(
            component_min <= component_max,
            "component_min must be <= component_max"
        );
        assert!(
            component_max < self.get_number_of_components(),
            "component index out of range"
        );

        let mut output =
            Vec::with_capacity((tuple_max - tuple_min + 1) * (component_max - component_min + 1));
        for tuple_idx in tuple_min..=tuple_max {
            let tuple = self.get_tuple(tuple_idx);
            output.extend_from_slice(&tuple[component_min..=component_max]);
        }
        output
    }

    /// VTK: `vtkDataArray::GetTuple`.
    pub(crate) fn tuple_as_f64(&self, tuple_idx: usize) -> Vec<f64> {
        self.get_tuple(tuple_idx)
    }

    pub fn get_range(&self) -> Option<[f64; 2]> {
        self.get_range_with_component(0)
    }

    /// VTK: `vtkDataArray::GetRange(int)`.
    pub(crate) fn get_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
        self.range_with_component_impl(component, false)
    }

    /// VTK: `vtkDataArray::GetFiniteRange(int)`.
    pub(crate) fn get_finite_range_with_component(&self, component: i32) -> Option<[f64; 2]> {
        self.range_with_component_impl(component, true)
    }

    /// VTK: `vtkDataArray::ComputeRange(double[2], int)`.
    pub(crate) fn compute_range(&self, range: &mut [f64], component: i32) {
        self.compute_range_with_ghosts(range, component, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeRange(double[2], int, const unsigned char*, unsigned char)`.
    pub(crate) fn compute_range_with_ghosts(
        &self,
        range: &mut [f64],
        component: i32,
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) {
        self.compute_component_or_vector_range_impl(
            range,
            component,
            ghosts,
            ghosts_to_skip,
            false,
        );
    }

    /// VTK: `vtkDataArray::ComputeFiniteRange(double[2], int)`.
    pub(crate) fn compute_finite_range(&self, range: &mut [f64], component: i32) {
        self.compute_finite_range_with_ghosts(range, component, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeFiniteRange(double[2], int, const unsigned char*, unsigned char)`.
    pub(crate) fn compute_finite_range_with_ghosts(
        &self,
        range: &mut [f64],
        component: i32,
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) {
        self.compute_component_or_vector_range_impl(range, component, ghosts, ghosts_to_skip, true);
    }

    /// VTK: `vtkDataArray::ComputeScalarRange(double*)`.
    pub fn compute_scalar_range(&self, ranges: &mut [f64]) -> bool {
        self.compute_scalar_range_with_ghosts(ranges, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeScalarRange(double*, const unsigned char*, unsigned char)`.
    pub fn compute_scalar_range_with_ghosts(
        &self,
        ranges: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) -> bool {
        self.compute_scalar_ranges_impl(ranges, ghosts, ghosts_to_skip, false)
    }

    /// VTK: `vtkDataArray::ComputeFiniteScalarRange(double*)`.
    pub fn compute_finite_scalar_range(&self, ranges: &mut [f64]) -> bool {
        self.compute_finite_scalar_range_with_ghosts(ranges, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeFiniteScalarRange(double*, const unsigned char*, unsigned char)`.
    pub fn compute_finite_scalar_range_with_ghosts(
        &self,
        ranges: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) -> bool {
        self.compute_scalar_ranges_impl(ranges, ghosts, ghosts_to_skip, true)
    }

    /// VTK: `vtkDataArray::ComputeVectorRange(double*)`.
    pub fn compute_vector_range(&self, range: &mut [f64]) -> bool {
        self.compute_vector_range_with_ghosts(range, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeVectorRange(double*, const unsigned char*, unsigned char)`.
    pub fn compute_vector_range_with_ghosts(
        &self,
        range: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) -> bool {
        self.compute_vector_ranges_impl(range, ghosts, ghosts_to_skip, false)
    }

    /// VTK: `vtkDataArray::ComputeFiniteVectorRange(double*)`.
    pub fn compute_finite_vector_range(&self, range: &mut [f64]) -> bool {
        self.compute_finite_vector_range_with_ghosts(range, None, 0xff)
    }

    /// VTK: `vtkDataArray::ComputeFiniteVectorRange(double*, const unsigned char*, unsigned char)`.
    pub fn compute_finite_vector_range_with_ghosts(
        &self,
        range: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
    ) -> bool {
        self.compute_vector_ranges_impl(range, ghosts, ghosts_to_skip, true)
    }

    fn compute_scalar_ranges_impl(
        &self,
        ranges: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
        finite_only: bool,
    ) -> bool {
        let number_of_components = self.get_number_of_components();
        assert!(
            ranges.len() >= number_of_components * 2,
            "ranges must hold two values per component"
        );
        self.assert_ghost_tuple_count(ghosts);

        let [range_min, range_max] = vtk_double_range_sentinels();
        for range in ranges[..number_of_components * 2].chunks_exact_mut(2) {
            range[0] = range_max;
            range[1] = range_min;
        }

        if self.get_number_of_tuples() == 0 {
            return false;
        }

        for (tuple_idx, tuple) in self.iter_tuples().enumerate() {
            if ghosts.is_some_and(|ghosts| ghosts[tuple_idx] & ghosts_to_skip != 0) {
                continue;
            }
            for (component_idx, item) in tuple.iter().enumerate() {
                let value = item.to_f64();
                let range = &mut ranges[component_idx * 2..component_idx * 2 + 2];
                let (min, max) = range.split_at_mut(1);
                update_range_vtk(&mut min[0], &mut max[0], value, finite_only);
            }
        }

        true
    }

    fn compute_vector_ranges_impl(
        &self,
        range: &mut [f64],
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
        finite_only: bool,
    ) -> bool {
        assert!(range.len() >= 2, "range must hold two values");
        self.assert_ghost_tuple_count(ghosts);

        let [range_min, range_max] = vtk_double_range_sentinels();
        let mut squared_min = range_max;
        let mut squared_max = range_min;

        range[0] = range_max;
        range[1] = range_min;

        if self.get_number_of_tuples() == 0 {
            return false;
        }

        for (tuple_idx, tuple) in self.iter_tuples().enumerate() {
            if ghosts.is_some_and(|ghosts| ghosts[tuple_idx] & ghosts_to_skip != 0) {
                continue;
            }

            let squared_sum = tuple
                .iter()
                .map(|item| {
                    let value = item.to_f64();
                    value * value
                })
                .sum::<f64>();

            update_range_vtk(&mut squared_min, &mut squared_max, squared_sum, finite_only);
        }

        range[0] = squared_min.sqrt();
        range[1] = squared_max.sqrt();
        true
    }

    fn assert_ghost_tuple_count(&self, ghosts: Option<&[u8]>) {
        if let Some(ghosts) = ghosts {
            assert!(
                ghosts.len() >= self.get_number_of_tuples(),
                "ghost array must have at least one entry per tuple"
            );
        }
    }

    fn compute_component_or_vector_range_impl(
        &self,
        range: &mut [f64],
        mut component: i32,
        ghosts: Option<&[u8]>,
        ghosts_to_skip: u8,
        finite_only: bool,
    ) {
        assert!(range.len() >= 2, "range must hold two values");
        if component >= self.get_number_of_components() as i32 {
            return;
        }
        if component < 0 && self.get_number_of_components() == 1 {
            component = 0;
        }

        let [range_min, range_max] = vtk_double_range_sentinels();
        range[0] = range_max;
        range[1] = range_min;

        if component < 0 {
            if finite_only {
                self.compute_finite_vector_range_with_ghosts(range, ghosts, ghosts_to_skip);
            } else {
                self.compute_vector_range_with_ghosts(range, ghosts, ghosts_to_skip);
            }
            return;
        }

        let number_of_components = self.get_number_of_components();
        let mut all_component_ranges = vec![0.0; number_of_components * 2];
        let computed = if finite_only {
            self.compute_finite_scalar_range_with_ghosts(
                &mut all_component_ranges,
                ghosts,
                ghosts_to_skip,
            )
        } else {
            self.compute_scalar_range_with_ghosts(&mut all_component_ranges, ghosts, ghosts_to_skip)
        };
        if computed {
            let offset = component as usize * 2;
            range[0] = all_component_ranges[offset];
            range[1] = all_component_ranges[offset + 1];
        }
    }

    fn range_with_component_impl(&self, component: i32, finite_only: bool) -> Option<[f64; 2]> {
        if component >= self.get_number_of_components() as i32 {
            return None;
        }
        let mut range = vtk_double_range_sentinels();
        if finite_only {
            self.compute_finite_range(&mut range, component);
        } else {
            self.compute_range(&mut range, component);
        }
        Some(range)
    }

    /// VTK: `vtkDataArray::GetMaxNorm`.
    pub fn get_max_norm(&self) -> f64 {
        let mut max_norm = 0.0;
        for tuple in self.iter_tuples() {
            let squared_sum = tuple
                .iter()
                .map(|value| {
                    let value = value.to_f64();
                    value * value
                })
                .sum::<f64>();
            let norm = squared_sum.sqrt();
            max_norm = if norm < max_norm { max_norm } else { norm };
        }
        max_norm
    }

    /// VTK: `vtkDataArray::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        (self.storage.capacity() * self.get_data_type_size()).div_ceil(1024)
    }

    /// VTK: `vtkDataArray::GetData` / `vtkAbstractArray::GetVoidPointer`.
    pub(crate) fn as_slice(&self) -> &[T] {
        self.storage.as_slice()
    }

    /// Mutable Rust equivalent of VTK raw data accessors.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        self.storage.as_mut_slice()
    }

    pub(crate) fn iter_tuples(&self) -> DataArrayTupleIter<'_, T, K> {
        DataArrayTupleIter {
            array: self,
            tuple_idx: 0,
        }
    }

    /// VTK: `vtkDataArray::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        self.storage.deep_copy(&other.storage);
    }

    /// VTK: `vtkDataArray::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.storage.shallow_copy(&other.storage);
    }

    pub(crate) fn deep_clone(&self) -> Self {
        Self {
            storage: self.storage.deep_clone(),
            kind: PhantomData,
        }
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            storage: self.storage.shallow_clone(),
            kind: PhantomData,
        }
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.storage.shares_storage_with(&other.storage)
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage.get_m_time()
    }
}

/// Iterator over VTK tuples.
pub(crate) struct DataArrayTupleIter<'a, T: Scalar, K: VtkArrayKind<T> = NativeVtkType<T>> {
    array: &'a DataArray<T, K>,
    tuple_idx: usize,
}

impl<'a, T: Scalar, K: VtkArrayKind<T>> Iterator for DataArrayTupleIter<'a, T, K> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.tuple_idx < self.array.get_number_of_tuples() {
            let tuple = self.array.get_typed_tuple(self.tuple_idx);
            self.tuple_idx += 1;
            Some(tuple)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.array.get_number_of_tuples() - self.tuple_idx;
        (remaining, Some(remaining))
    }
}

impl<T: Scalar, K: VtkArrayKind<T>> ExactSizeIterator for DataArrayTupleIter<'_, T, K> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_component_grows_complete_default_tuples() {
        let mut array = DataArray::<f64>::with_name_and_number_of_components("vectors", 3);

        array.insert_component(2, 1, 7.0);

        assert_eq!(array.get_number_of_values(), 9);
        assert_eq!(array.get_number_of_tuples(), 3);
        assert_eq!(array.get_component(2, 1), 7.0);
        assert_eq!(
            array.as_slice(),
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0, 0.0]
        );
    }

    #[test]
    fn set_component_grows_complete_default_tuples() {
        let mut array = DataArray::<i32>::with_name_and_number_of_components("vectors", 2);

        array.set_component(2, 0, 9.0);

        assert_eq!(array.get_number_of_values(), 6);
        assert_eq!(array.get_number_of_tuples(), 3);
        assert_eq!(array.as_slice(), &[0, 0, 0, 0, 9, 0]);
    }

    #[test]
    fn set_component_preserves_existing_tuple_components() {
        let mut array = DataArray::<f64>::from_vec("vectors", vec![1.0f64, 2.0, 3.0, 4.0], 2);

        array.set_component(1, 0, 9.0);

        assert_eq!(array.as_slice(), &[1.0, 2.0, 9.0, 4.0]);
    }

    #[test]
    fn insert_and_get_tuples_copy_flat_components() {
        let mut array = DataArray::<i32>::with_name_and_number_of_components("pairs", 2);
        array.insert_next_typed_tuple(&[1, 2]);
        array.insert_next_typed_tuple(&[3, 4]);
        array.insert_typed_tuple(4, &[9, 10]);

        assert_eq!(array.get_number_of_tuples(), 5);
        let mut output = DataArray::<i32>::with_name_and_number_of_components("out", 2);
        output.set_number_of_tuples(2);
        array.get_tuples(&[1, 4], &mut output);
        assert_eq!(output.as_slice(), &[3, 4, 9, 10]);
        assert_eq!(array.get_data(1, 4, 0, 0), vec![3.0, 0.0, 0.0, 9.0]);

        output.insert_tuples_by_ids(&[1, 3], &[0, 1], &array);
        assert_eq!(output.as_slice(), &[3, 4, 1, 2, 0, 0, 3, 4]);
        output.insert_tuples_starting_at(0, &[4, 0], &array);
        assert_eq!(output.as_slice(), &[9, 10, 1, 2, 0, 0, 3, 4]);
    }

    #[test]
    fn compute_range_supports_components_norms_and_finite_values() {
        let array =
            DataArray::<f64>::from_vec("vectors", vec![3.0f64, 4.0, 0.0, 5.0, 12.0, 0.0], 3);

        let mut range = [0.0, 0.0];
        array.compute_range(&mut range, 0);
        assert_eq!(range, [3.0, 5.0]);
        array.compute_range(&mut range, -1);
        assert_eq!(range[1], 13.0);

        let with_nan = DataArray::<f64>::from_vec("values", vec![f64::NAN, 100.0], 1);
        with_nan.compute_finite_range(&mut range, 0);
        assert_eq!(range, [100.0, 100.0]);
    }

    #[test]
    fn fill_component_and_fill_mutate_existing_values() {
        let mut array = DataArray::<f32>::from_vec("vectors", vec![1.0f32, 2.0, 3.0, 4.0], 2);

        array.fill_component(1, 8.0);
        assert_eq!(array.as_slice(), &[1.0, 8.0, 3.0, 8.0]);

        array.fill(5.0);
        assert_eq!(array.as_slice(), &[5.0, 5.0, 5.0, 5.0]);
    }

    #[test]
    fn deep_copy_clones_and_shallow_copy_shares_until_mutation() {
        let source = DataArray::from_vec("source", vec![1u16, 2, 3, 4], 2);
        let mut deep = DataArray::<u16>::with_name_and_number_of_components("deep", 1);
        let mut shallow = DataArray::<u16>::with_name_and_number_of_components("shallow", 1);

        deep.deep_copy(&source);
        shallow.shallow_copy(&source);

        assert_eq!(deep, source);
        assert_eq!(shallow, source);

        deep.set_component(0, 0, 9.0);
        assert_eq!(source.get_component(0, 0), 1.0);

        shallow.set_component(0, 0, 8.0);
        assert_eq!(source.get_component(0, 0), 1.0);
        assert!(!shallow.shares_storage_with(&source));
    }
}
