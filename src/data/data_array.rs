use crate::types::{Scalar, ScalarType};
use std::sync::Arc;

/// A contiguous array of tuples, where each tuple has `num_components` values.
///
/// This is the fundamental data container, analogous to VTK's `vtkDataArray`.
/// Uses `Arc<Vec<T>>` for zero-copy clone with copy-on-write semantics.
///
/// # Examples
///
/// ```
/// use vtk_pure_rs::data::DataArray;
///
/// // Create a 3-component array (e.g., for normals or positions)
/// let mut normals = DataArray::<f64>::new("Normals", 3);
/// normals.push_tuple(&[0.0, 0.0, 1.0]);
/// normals.push_tuple(&[0.0, 1.0, 0.0]);
/// assert_eq!(normals.num_tuples(), 2);
///
/// // Create from a Vec
/// let scalars = DataArray::<f64>::from_vec("Temperature", vec![10.0, 20.0, 30.0], 1);
/// assert_eq!(scalars.num_tuples(), 3);
/// ```
#[derive(Debug)]
pub struct DataArray<T: Scalar> {
    data: Arc<Vec<T>>,
    num_components: usize,
    name: String,
}

impl<T: Scalar> Clone for DataArray<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            num_components: self.num_components,
            name: self.name.clone(),
        }
    }
}

impl<T: Scalar> PartialEq for DataArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.num_components == other.num_components
            && self.name == other.name
            && (Arc::ptr_eq(&self.data, &other.data) || *self.data == *other.data)
    }
}

impl<T: Scalar> DataArray<T> {
    pub fn new(name: impl Into<String>, num_components: usize) -> Self {
        assert!(num_components > 0, "num_components must be > 0");
        Self {
            data: Arc::new(Vec::new()),
            num_components,
            name: name.into(),
        }
    }

    pub fn from_vec(name: impl Into<String>, data: Vec<T>, num_components: usize) -> Self {
        assert!(num_components > 0, "num_components must be > 0");
        assert!(
            data.len().is_multiple_of(num_components),
            "data length {} is not divisible by num_components {}",
            data.len(),
            num_components
        );
        Self {
            data: Arc::new(data),
            num_components,
            name: name.into(),
        }
    }

    /// Create from a borrowed slice (copies data).
    pub fn from_slice(name: impl Into<String>, data: &[T], num_components: usize) -> Self {
        Self::from_vec(name, data.to_vec(), num_components)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn scalar_type(&self) -> ScalarType {
        T::TYPE_ID
    }

    pub fn num_components(&self) -> usize {
        self.num_components
    }

    pub fn num_tuples(&self) -> usize {
        self.data.len() / self.num_components
    }

    pub fn tuple(&self, idx: usize) -> &[T] {
        let start = idx * self.num_components;
        &self.data[start..start + self.num_components]
    }

    pub fn tuple_mut(&mut self, idx: usize) -> &mut [T] {
        let start = idx * self.num_components;
        let nc = self.num_components;
        // Arc::make_mut is already optimal: no-op when strong_count == 1
        let data = Arc::make_mut(&mut self.data);
        &mut data[start..start + nc]
    }

    pub fn push_tuple(&mut self, values: &[T]) {
        assert_eq!(
            values.len(),
            self.num_components,
            "expected {} components, got {}",
            self.num_components,
            values.len()
        );
        // Fast path: if we're the sole owner, avoid Arc::make_mut's atomic CAS
        if let Some(v) = Arc::get_mut(&mut self.data) {
            v.extend_from_slice(values);
        } else {
            Arc::make_mut(&mut self.data).extend_from_slice(values);
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        Arc::make_mut(&mut self.data).as_mut_slice()
    }

    pub fn into_vec(self) -> Vec<T> {
        Arc::try_unwrap(self.data).unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Create a 1-component DataArray by evaluating a function for each index.
    pub fn from_fn(name: impl Into<String>, count: usize, f: impl Fn(usize) -> T) -> Self {
        let data: Vec<T> = (0..count).map(f).collect();
        Self::from_vec(name, data, 1)
    }

    /// Create a multi-component DataArray from a function returning a slice.
    pub fn from_fn_components(
        name: impl Into<String>,
        count: usize,
        num_components: usize,
        f: impl Fn(usize) -> Vec<T>,
    ) -> Self {
        let mut data = Vec::with_capacity(count * num_components);
        for i in 0..count {
            let vals = f(i);
            assert_eq!(vals.len(), num_components);
            data.extend(vals);
        }
        Self::from_vec(name, data, num_components)
    }

    /// Create a DataArray filled with a constant value.
    pub fn filled(name: impl Into<String>, value: T, count: usize, num_components: usize) -> Self {
        Self::from_vec(name, vec![value; count * num_components], num_components)
    }

    /// Number of total scalar values (tuples * components).
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Apply a function to each scalar value in-place.
    pub fn map_in_place(&mut self, f: impl Fn(T) -> T) {
        for v in Arc::make_mut(&mut self.data).iter_mut() {
            *v = f(*v);
        }
    }

    /// Create a new DataArray by mapping each scalar value.
    pub fn map(&self, name: impl Into<String>, f: impl Fn(T) -> T) -> Self {
        let mapped: Vec<T> = self.data.iter().map(|&v| f(v)).collect();
        Self::from_vec(name, mapped, self.num_components)
    }

    /// Create a new 1-component DataArray from a per-tuple function.
    pub fn map_tuples(&self, name: impl Into<String>, f: impl Fn(&[T]) -> T) -> DataArray<T> {
        let mut result = DataArray::new(name, 1);
        for i in 0..self.num_tuples() {
            let t = self.tuple(i);
            result.push_tuple(&[f(t)]);
        }
        result
    }

    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.data).clear();
    }

    /// Returns true if this array shares storage with another clone.
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    /// Ensure exclusive ownership. Call before tight mutation loops to
    /// avoid per-call Arc::make_mut atomic checks.
    pub fn make_unique(&mut self) {
        Arc::make_mut(&mut self.data);
    }

    /// Iterate over tuples as slices.
    pub fn iter_tuples(&self) -> DataArrayTupleIter<'_, T> {
        DataArrayTupleIter {
            array: self,
            idx: 0,
        }
    }
}

/// Index by tuple index, returns the tuple slice.
impl<T: Scalar> std::ops::Index<usize> for DataArray<T> {
    type Output = [T];

    fn index(&self, idx: usize) -> &Self::Output {
        self.tuple(idx)
    }
}

/// Iterator over tuples in a DataArray.
pub struct DataArrayTupleIter<'a, T: Scalar> {
    array: &'a DataArray<T>,
    idx: usize,
}

impl<'a, T: Scalar> Iterator for DataArrayTupleIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.array.num_tuples() {
            let t = self.array.tuple(self.idx);
            self.idx += 1;
            Some(t)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let r = self.array.num_tuples() - self.idx;
        (r, Some(r))
    }
}

impl<T: Scalar> ExactSizeIterator for DataArrayTupleIter<'_, T> {}

impl DataArray<f64> {
    /// Scale all values by a factor.
    pub fn scale(&mut self, factor: f64) {
        for v in Arc::make_mut(&mut self.data).iter_mut() {
            *v *= factor;
        }
    }

    /// Normalize all values to [0, 1] range.
    /// Returns the original (min, max) range.
    pub fn normalize(&mut self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &v in self.data.iter() {
            min = min.min(v);
            max = max.max(v);
        }
        let range = max - min;
        if range > 1e-15 {
            for v in Arc::make_mut(&mut self.data).iter_mut() {
                *v = (*v - min) / range;
            }
        }
        (min, max)
    }

    /// Compute min value (first component only for multi-component).
    pub fn min_value(&self) -> f64 {
        let mut min = f64::INFINITY;
        for i in 0..self.num_tuples() {
            min = min.min(self.tuple(i)[0]);
        }
        min
    }

    /// Compute max value (first component only for multi-component).
    pub fn max_value(&self) -> f64 {
        let mut max = f64::NEG_INFINITY;
        for i in 0..self.num_tuples() {
            max = max.max(self.tuple(i)[0]);
        }
        max
    }

    /// Compute magnitude for each tuple (Euclidean norm of components).
    pub fn magnitude(&self, name: impl Into<String>) -> DataArray<f64> {
        let mut result = DataArray::new(name, 1);
        for i in 0..self.num_tuples() {
            let t = self.tuple(i);
            let mag: f64 = t.iter().map(|v| v * v).sum::<f64>().sqrt();
            result.push_tuple(&[mag]);
        }
        result
    }
}

impl DataArray<f64> {
    /// Extract a single component as a new 1-component array.
    pub fn extract_component(&self, component: usize, name: impl Into<String>) -> DataArray<f64> {
        assert!(
            component < self.num_components,
            "component index out of range"
        );
        let mut result = DataArray::new(name, 1);
        for i in 0..self.num_tuples() {
            result.push_tuple(&[self.tuple(i)[component]]);
        }
        result
    }

    /// Concatenate two arrays with the same number of components.
    pub fn concat(&self, other: &DataArray<f64>) -> DataArray<f64> {
        assert_eq!(
            self.num_components, other.num_components,
            "cannot concatenate arrays with different component counts"
        );
        let mut data = (*self.data).clone();
        data.extend_from_slice(&other.data);
        DataArray::from_vec(self.name(), data, self.num_components)
    }

    /// Append another array's tuples to this array.
    pub fn extend(&mut self, other: &DataArray<f64>) {
        assert_eq!(self.num_components, other.num_components);
        Arc::make_mut(&mut self.data).extend_from_slice(&other.data);
    }

    /// Create a new array by combining components from multiple 1-component arrays.
    pub fn compose(name: impl Into<String>, components: &[&DataArray<f64>]) -> DataArray<f64> {
        assert!(!components.is_empty());
        let nt = components[0].num_tuples();
        for c in components {
            assert_eq!(
                c.num_tuples(),
                nt,
                "all components must have same tuple count"
            );
            assert_eq!(c.num_components(), 1, "all inputs must be 1-component");
        }
        let nc = components.len();
        let mut data = Vec::with_capacity(nt * nc);
        for i in 0..nt {
            for c in components {
                data.push(c.tuple(i)[0]);
            }
        }
        DataArray::from_vec(name, data, nc)
    }
}

impl DataArray<f32> {
    /// Scale all values by a factor.
    pub fn scale(&mut self, factor: f32) {
        for v in Arc::make_mut(&mut self.data).iter_mut() {
            *v *= factor;
        }
    }
}

/// Type-erased data array that can hold any scalar type.
///
/// Uses an enum rather than trait objects for exhaustive matching and zero vtable overhead.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyDataArray {
    F32(DataArray<f32>),
    F64(DataArray<f64>),
    I8(DataArray<i8>),
    I16(DataArray<i16>),
    I32(DataArray<i32>),
    I64(DataArray<i64>),
    U8(DataArray<u8>),
    U16(DataArray<u16>),
    U32(DataArray<u32>),
    U64(DataArray<u64>),
}

impl AnyDataArray {
    pub fn name(&self) -> &str {
        match self {
            AnyDataArray::F32(a) => a.name(),
            AnyDataArray::F64(a) => a.name(),
            AnyDataArray::I8(a) => a.name(),
            AnyDataArray::I16(a) => a.name(),
            AnyDataArray::I32(a) => a.name(),
            AnyDataArray::I64(a) => a.name(),
            AnyDataArray::U8(a) => a.name(),
            AnyDataArray::U16(a) => a.name(),
            AnyDataArray::U32(a) => a.name(),
            AnyDataArray::U64(a) => a.name(),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        match self {
            AnyDataArray::F32(a) => a.set_name(name),
            AnyDataArray::F64(a) => a.set_name(name),
            AnyDataArray::I8(a) => a.set_name(name),
            AnyDataArray::I16(a) => a.set_name(name),
            AnyDataArray::I32(a) => a.set_name(name),
            AnyDataArray::I64(a) => a.set_name(name),
            AnyDataArray::U8(a) => a.set_name(name),
            AnyDataArray::U16(a) => a.set_name(name),
            AnyDataArray::U32(a) => a.set_name(name),
            AnyDataArray::U64(a) => a.set_name(name),
        }
    }

    pub fn scalar_type(&self) -> ScalarType {
        match self {
            AnyDataArray::F32(_) => ScalarType::F32,
            AnyDataArray::F64(_) => ScalarType::F64,
            AnyDataArray::I8(_) => ScalarType::I8,
            AnyDataArray::I16(_) => ScalarType::I16,
            AnyDataArray::I32(_) => ScalarType::I32,
            AnyDataArray::I64(_) => ScalarType::I64,
            AnyDataArray::U8(_) => ScalarType::U8,
            AnyDataArray::U16(_) => ScalarType::U16,
            AnyDataArray::U32(_) => ScalarType::U32,
            AnyDataArray::U64(_) => ScalarType::U64,
        }
    }

    /// Clone this array with a different name.
    pub fn clone_with_name(&self, name: &str) -> Self {
        let mut cloned = self.clone();
        cloned.set_name(name);
        cloned
    }

    pub fn num_components(&self) -> usize {
        match self {
            AnyDataArray::F32(a) => a.num_components(),
            AnyDataArray::F64(a) => a.num_components(),
            AnyDataArray::I8(a) => a.num_components(),
            AnyDataArray::I16(a) => a.num_components(),
            AnyDataArray::I32(a) => a.num_components(),
            AnyDataArray::I64(a) => a.num_components(),
            AnyDataArray::U8(a) => a.num_components(),
            AnyDataArray::U16(a) => a.num_components(),
            AnyDataArray::U32(a) => a.num_components(),
            AnyDataArray::U64(a) => a.num_components(),
        }
    }

    pub fn num_tuples(&self) -> usize {
        match self {
            AnyDataArray::F32(a) => a.num_tuples(),
            AnyDataArray::F64(a) => a.num_tuples(),
            AnyDataArray::I8(a) => a.num_tuples(),
            AnyDataArray::I16(a) => a.num_tuples(),
            AnyDataArray::I32(a) => a.num_tuples(),
            AnyDataArray::I64(a) => a.num_tuples(),
            AnyDataArray::U8(a) => a.num_tuples(),
            AnyDataArray::U16(a) => a.num_tuples(),
            AnyDataArray::U32(a) => a.num_tuples(),
            AnyDataArray::U64(a) => a.num_tuples(),
        }
    }

    /// Get tuple values as f64 (type-erased access).
    pub fn tuple_as_f64(&self, idx: usize, out: &mut [f64]) {
        macro_rules! convert {
            ($arr:expr) => {{
                let t = $arr.tuple(idx);
                for (o, v) in out.iter_mut().zip(t.iter()) {
                    *o = v.to_f64();
                }
            }};
        }
        match self {
            AnyDataArray::F32(a) => convert!(a),
            AnyDataArray::F64(a) => convert!(a),
            AnyDataArray::I8(a) => convert!(a),
            AnyDataArray::I16(a) => convert!(a),
            AnyDataArray::I32(a) => convert!(a),
            AnyDataArray::I64(a) => convert!(a),
            AnyDataArray::U8(a) => convert!(a),
            AnyDataArray::U16(a) => convert!(a),
            AnyDataArray::U32(a) => convert!(a),
            AnyDataArray::U64(a) => convert!(a),
        }
    }

    /// Compute statistics (min, max, mean, variance) of the first component.
    ///
    /// Returns `(min, max, mean, variance)`. Returns `None` if the array is empty.
    pub fn statistics(&self) -> Option<ArrayStatistics> {
        let nt = self.num_tuples();
        if nt == 0 {
            return None;
        }
        let mut buf = [0.0f64];
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for i in 0..nt {
            self.tuple_as_f64(i, &mut buf);
            let v = buf[0];
            min = min.min(v);
            max = max.max(v);
            sum += v;
            sum_sq += v * v;
        }
        let mean = sum / nt as f64;
        let variance = sum_sq / nt as f64 - mean * mean;
        Some(ArrayStatistics {
            min,
            max,
            mean,
            variance: variance.max(0.0),
            count: nt,
        })
    }

    /// Compute the range [min, max] of the first component.
    pub fn range(&self) -> Option<[f64; 2]> {
        self.statistics().map(|s| [s.min, s.max])
    }

    /// Extract all values as a Vec<f64> (first component of each tuple).
    ///
    /// Convenient for analysis workflows. For multi-component arrays,
    /// only the first component is returned.
    pub fn to_f64_vec(&self) -> Vec<f64> {
        let nt = self.num_tuples();
        let mut result = Vec::with_capacity(nt);
        let mut buf = [0.0f64];
        for i in 0..nt {
            self.tuple_as_f64(i, &mut buf);
            result.push(buf[0]);
        }
        result
    }

    /// Extract all values as a flat Vec<f64> (all components).
    pub fn to_f64_vec_flat(&self) -> Vec<f64> {
        let nt = self.num_tuples();
        let nc = self.num_components();
        let mut result = Vec::with_capacity(nt * nc);
        let mut buf = vec![0.0f64; nc];
        for i in 0..nt {
            self.tuple_as_f64(i, &mut buf);
            result.extend_from_slice(&buf);
        }
        result
    }

    /// Try to get the inner DataArray<f64> if this is the F64 variant.
    pub fn as_f64(&self) -> Option<&DataArray<f64>> {
        match self {
            AnyDataArray::F64(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get the inner DataArray<f32> if this is the F32 variant.
    pub fn as_f32(&self) -> Option<&DataArray<f32>> {
        match self {
            AnyDataArray::F32(a) => Some(a),
            _ => None,
        }
    }
}

impl std::fmt::Display for AnyDataArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = match self.scalar_type() {
            ScalarType::F32 => "Float32",
            ScalarType::F64 => "Float64",
            ScalarType::I8 => "Int8",
            ScalarType::I16 => "Int16",
            ScalarType::I32 => "Int32",
            ScalarType::I64 => "Int64",
            ScalarType::U8 => "UInt8",
            ScalarType::U16 => "UInt16",
            ScalarType::U32 => "UInt32",
            ScalarType::U64 => "UInt64",
        };
        write!(
            f,
            "DataArray<{}>(\"{}\", {} tuples, {} components)",
            type_name,
            self.name(),
            self.num_tuples(),
            self.num_components()
        )
    }
}

/// Statistics computed from a data array.
#[derive(Debug, Clone, Copy)]
pub struct ArrayStatistics {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
}

impl ArrayStatistics {
    /// Standard deviation.
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }
}

// Specific From impls for each scalar type.
macro_rules! impl_from_data_array {
    ($ty:ty, $variant:ident) => {
        impl From<DataArray<$ty>> for AnyDataArray {
            fn from(a: DataArray<$ty>) -> Self {
                AnyDataArray::$variant(a)
            }
        }
    };
}

impl_from_data_array!(f32, F32);
impl_from_data_array!(f64, F64);
impl_from_data_array!(i8, I8);
impl_from_data_array!(i16, I16);
impl_from_data_array!(i32, I32);
impl_from_data_array!(i64, I64);
impl_from_data_array!(u8, U8);
impl_from_data_array!(u16, U16);
impl_from_data_array!(u32, U32);
impl_from_data_array!(u64, U64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_array_basics() {
        let mut arr = DataArray::<f64>::new("test", 3);
        arr.push_tuple(&[1.0, 2.0, 3.0]);
        arr.push_tuple(&[4.0, 5.0, 6.0]);
        assert_eq!(arr.num_tuples(), 2);
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.tuple(0), &[1.0, 2.0, 3.0]);
        assert_eq!(arr.tuple(1), &[4.0, 5.0, 6.0]);
    }

    #[test]
    #[should_panic(expected = "expected 3 components")]
    fn push_tuple_rejects_wrong_component_count() {
        let mut arr = DataArray::<f64>::new("vectors", 3);
        arr.push_tuple(&[1.0, 2.0]);
    }

    #[test]
    fn data_array_from_vec() {
        let arr = DataArray::from_vec("coords", vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        assert_eq!(arr.num_tuples(), 2);
        assert_eq!(arr.scalar_type(), ScalarType::F32);
    }

    #[test]
    fn any_data_array_type_erased() {
        let arr = DataArray::from_vec("scalars", vec![10i32, 20, 30], 1);
        let any: AnyDataArray = arr.into();
        assert_eq!(any.name(), "scalars");
        assert_eq!(any.scalar_type(), ScalarType::I32);
        assert_eq!(any.num_tuples(), 3);

        let mut val = [0.0f64];
        any.tuple_as_f64(1, &mut val);
        assert_eq!(val[0], 20.0);
    }

    #[test]
    fn statistics_basic() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "test",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            1,
        ));
        let stats = arr.statistics().unwrap();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert!((stats.mean - 3.0).abs() < 1e-12);
        assert_eq!(stats.count, 5);
        assert!(stats.variance > 0.0);
        assert!((stats.std_dev() - 2.0f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn statistics_empty() {
        let arr = AnyDataArray::F64(DataArray::new("test", 1));
        assert!(arr.statistics().is_none());
    }

    #[test]
    fn range() {
        let arr = AnyDataArray::I32(DataArray::from_vec("test", vec![10, -5, 20, 0], 1));
        let r = arr.range().unwrap();
        assert_eq!(r[0], -5.0);
        assert_eq!(r[1], 20.0);
    }

    #[test]
    fn map_in_place() {
        let mut arr = DataArray::from_vec("test", vec![1.0f64, 2.0, 3.0], 1);
        arr.map_in_place(|v| v * 2.0);
        assert_eq!(arr.tuple(0), &[2.0]);
        assert_eq!(arr.tuple(2), &[6.0]);
    }

    #[test]
    fn map_new() {
        let arr = DataArray::from_vec("in", vec![1.0f64, 4.0, 9.0], 1);
        let sqrt = arr.map("sqrt", |v| v.sqrt());
        assert_eq!(sqrt.name(), "sqrt");
        assert!((sqrt.tuple(1)[0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn map_tuples_magnitude() {
        let arr = DataArray::from_vec("vec", vec![3.0f64, 4.0, 0.0], 3);
        let mag = arr.map_tuples("mag", |t| (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt());
        assert!((mag.tuple(0)[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn scale_f64() {
        let mut arr = DataArray::from_vec("test", vec![1.0f64, 2.0, 3.0], 1);
        arr.scale(10.0);
        assert_eq!(arr.tuple(0), &[10.0]);
        assert_eq!(arr.tuple(2), &[30.0]);
    }

    #[test]
    fn normalize_f64() {
        let mut arr = DataArray::from_vec("test", vec![0.0f64, 50.0, 100.0], 1);
        let (min, max) = arr.normalize();
        assert_eq!(min, 0.0);
        assert_eq!(max, 100.0);
        assert!((arr.tuple(0)[0]).abs() < 1e-12);
        assert!((arr.tuple(1)[0] - 0.5).abs() < 1e-12);
        assert!((arr.tuple(2)[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn min_max_value() {
        let arr = DataArray::from_vec("test", vec![5.0f64, -3.0, 8.0, 1.0], 1);
        assert_eq!(arr.min_value(), -3.0);
        assert_eq!(arr.max_value(), 8.0);
    }

    #[test]
    fn magnitude_3d() {
        let arr = DataArray::from_vec("v", vec![1.0f64, 0.0, 0.0, 0.0, 3.0, 4.0], 3);
        let mag = arr.magnitude("mag");
        assert!((mag.tuple(0)[0] - 1.0).abs() < 1e-12);
        assert!((mag.tuple(1)[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn extract_component() {
        let arr = DataArray::from_vec("vec", vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        let y = arr.extract_component(1, "y");
        assert_eq!(y.num_tuples(), 2);
        assert_eq!(y.num_components(), 1);
        assert_eq!(y.tuple(0), &[2.0]);
        assert_eq!(y.tuple(1), &[5.0]);
    }

    #[test]
    fn concat_arrays() {
        let a = DataArray::from_vec("a", vec![1.0f64, 2.0], 1);
        let b = DataArray::from_vec("b", vec![3.0f64, 4.0], 1);
        let c = a.concat(&b);
        assert_eq!(c.num_tuples(), 4);
        assert_eq!(c.tuple(2), &[3.0]);
    }

    #[test]
    fn extend_array() {
        let mut a = DataArray::from_vec("a", vec![1.0f64, 2.0], 1);
        let b = DataArray::from_vec("b", vec![3.0f64, 4.0], 1);
        a.extend(&b);
        assert_eq!(a.num_tuples(), 4);
    }

    #[test]
    fn compose_components() {
        let x = DataArray::from_vec("x", vec![1.0f64, 4.0], 1);
        let y = DataArray::from_vec("y", vec![2.0f64, 5.0], 1);
        let z = DataArray::from_vec("z", vec![3.0f64, 6.0], 1);
        let v = DataArray::compose("vec", &[&x, &y, &z]);
        assert_eq!(v.num_tuples(), 2);
        assert_eq!(v.num_components(), 3);
        assert_eq!(v.tuple(0), &[1.0, 2.0, 3.0]);
        assert_eq!(v.tuple(1), &[4.0, 5.0, 6.0]);
    }

    #[test]
    fn from_fn() {
        let arr = DataArray::<f64>::from_fn("sq", 5, |i| (i * i) as f64);
        assert_eq!(arr.num_tuples(), 5);
        assert_eq!(arr.tuple(3), &[9.0]);
    }

    #[test]
    fn from_fn_components() {
        let arr = DataArray::<f64>::from_fn_components("pos", 3, 3, |i| {
            vec![i as f64, (i * 2) as f64, (i * 3) as f64]
        });
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.tuple(1), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn filled() {
        let arr = DataArray::<f64>::filled("zeros", 0.0, 10, 1);
        assert_eq!(arr.num_tuples(), 10);
        assert_eq!(arr.tuple(5), &[0.0]);
    }

    #[test]
    fn iter_tuples() {
        let arr = DataArray::<f64>::from_vec("test", vec![1.0, 2.0, 3.0], 1);
        let vals: Vec<&[f64]> = arr.iter_tuples().collect();
        assert_eq!(vals.len(), 3);
        assert_eq!(vals[1], &[2.0]);
    }

    #[test]
    fn any_data_array_display() {
        let arr = AnyDataArray::F64(DataArray::from_vec("temperature", vec![1.0, 2.0, 3.0], 1));
        let s = format!("{arr}");
        assert!(s.contains("Float64"));
        assert!(s.contains("temperature"));
        assert!(s.contains("3 tuples"));
    }

    #[test]
    fn to_f64_vec() {
        let arr = AnyDataArray::F64(DataArray::from_vec("t", vec![1.0, 2.0, 3.0], 1));
        let v = arr.to_f64_vec();
        assert_eq!(v, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn to_f64_vec_flat() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "v",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            3,
        ));
        let v = arr.to_f64_vec_flat();
        assert_eq!(v, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn as_f64_variant() {
        let arr = AnyDataArray::F64(DataArray::from_vec("t", vec![1.0], 1));
        assert!(arr.as_f64().is_some());
        assert!(arr.as_f32().is_none());
    }

    #[test]
    fn index_trait() {
        let arr = DataArray::from_vec("v", vec![1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        assert_eq!(arr[0], [1.0, 2.0, 3.0]);
        assert_eq!(arr[1], [4.0, 5.0, 6.0]);
    }
}
