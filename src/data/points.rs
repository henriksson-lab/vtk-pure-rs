use crate::types::{BoundingBox, Scalar};

use crate::data::DataArray;

/// A collection of 3D points, backed by a 3-component `DataArray`.
///
/// Default precision is `f64`, matching VTK convention.
#[derive(Debug, Clone, PartialEq)]
pub struct Points<T: Scalar = f64> {
    data: DataArray<T>,
}

impl<T: Scalar> Points<T> {
    pub fn new() -> Self {
        Self {
            data: DataArray::new("Points", 3),
        }
    }

    pub fn with_capacity(_capacity: usize) -> Self {
        // DataArray doesn't expose reserve yet, so just create empty for now.
        Self::new()
    }

    pub fn from_vec(points: Vec<[T; 3]>) -> Self {
        let flat: Vec<T> = points.into_iter().flatten().collect();
        Self {
            data: DataArray::from_vec("Points", flat, 3),
        }
    }

    /// Create from a pre-flattened Vec (x0,y0,z0,x1,y1,z1,...). Zero-copy.
    pub fn from_flat_vec(flat: Vec<T>) -> Self {
        debug_assert!(
            flat.len() % 3 == 0,
            "flat vec length must be divisible by 3"
        );
        Self {
            data: DataArray::from_vec("Points", flat, 3),
        }
    }

    pub fn push(&mut self, point: [T; 3]) {
        self.data.push_tuple(&point);
    }

    pub fn get(&self, idx: usize) -> [T; 3] {
        let t = self.data.tuple(idx);
        [t[0], t[1], t[2]]
    }

    pub fn set(&mut self, idx: usize, point: [T; 3]) {
        let t = self.data.tuple_mut(idx);
        t[0] = point[0];
        t[1] = point[1];
        t[2] = point[2];
    }

    pub fn len(&self) -> usize {
        self.data.num_tuples()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_data_array(&self) -> &DataArray<T> {
        &self.data
    }

    pub fn into_data_array(self) -> DataArray<T> {
        self.data
    }

    /// Access the raw flat data as a slice: [x0, y0, z0, x1, y1, z1, ...].
    ///
    /// Useful for GPU upload, FFI, or numpy interop.
    pub fn as_flat_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    /// Access the raw flat data as a mutable slice.
    pub fn as_flat_slice_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for i in 0..self.len() {
            let p = self.get(i);
            bb.expand([p[0].to_f64(), p[1].to_f64(), p[2].to_f64()]);
        }
        bb
    }

    /// Create points from a function that maps index to position.
    pub fn from_fn(count: usize, f: impl Fn(usize) -> [T; 3]) -> Self {
        let mut pts = Self::new();
        for i in 0..count {
            pts.push(f(i));
        }
        pts
    }

    /// Collect all points into a Vec of arrays.
    pub fn to_vec(&self) -> Vec<[T; 3]> {
        (0..self.len()).map(|i| self.get(i)).collect()
    }
}

impl Points<f64> {
    /// Compute the centroid (average position).
    pub fn centroid(&self) -> [f64; 3] {
        if self.is_empty() {
            return [0.0; 3];
        }
        let n = self.len() as f64;
        let mut c = [0.0; 3];
        for p in self {
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        [c[0] / n, c[1] / n, c[2] / n]
    }

    /// Transform all points in-place by a function.
    pub fn transform(&mut self, f: impl Fn([f64; 3]) -> [f64; 3]) {
        for i in 0..self.len() {
            let p = self.get(i);
            self.set(i, f(p));
        }
    }
}

/// Iterator over points, yielding `[T; 3]`.
pub struct PointsIter<'a, T: Scalar> {
    points: &'a Points<T>,
    idx: usize,
}

impl<'a, T: Scalar> Iterator for PointsIter<'a, T> {
    type Item = [T; 3];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.points.len() {
            let p = self.points.get(self.idx);
            self.idx += 1;
            Some(p)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.points.len() - self.idx;
        (remaining, Some(remaining))
    }
}

impl<'a, T: Scalar> ExactSizeIterator for PointsIter<'a, T> {}

impl<T: Scalar> Points<T> {
    /// Iterate over all points.
    pub fn iter(&self) -> PointsIter<'_, T> {
        PointsIter {
            points: self,
            idx: 0,
        }
    }
}

impl<'a, T: Scalar> IntoIterator for &'a Points<T> {
    type Item = [T; 3];
    type IntoIter = PointsIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Scalar> std::ops::Index<usize> for Points<T> {
    type Output = [T];

    fn index(&self, idx: usize) -> &Self::Output {
        self.data.tuple(idx)
    }
}

impl<T: Scalar> FromIterator<[T; 3]> for Points<T> {
    fn from_iter<I: IntoIterator<Item = [T; 3]>>(iter: I) -> Self {
        let mut pts = Self::new();
        for p in iter {
            pts.push(p);
        }
        pts
    }
}

impl<T: Scalar> From<Vec<[T; 3]>> for Points<T> {
    fn from(v: Vec<[T; 3]>) -> Self {
        Self::from_vec(v)
    }
}

impl<T: Scalar> Extend<[T; 3]> for Points<T> {
    fn extend<I: IntoIterator<Item = [T; 3]>>(&mut self, iter: I) {
        for p in iter {
            self.push(p);
        }
    }
}

impl<T: Scalar> Default for Points<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_get() {
        let mut pts = Points::<f64>::new();
        pts.push([1.0, 2.0, 3.0]);
        pts.push([4.0, 5.0, 6.0]);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts.get(0), [1.0, 2.0, 3.0]);
        assert_eq!(pts.get(1), [4.0, 5.0, 6.0]);
    }

    #[test]
    fn from_vec_points() {
        let pts = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts.get(1), [1.0, 1.0, 1.0]);
    }

    #[test]
    fn iterator() {
        let pts = Points::from_vec(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let collected: Vec<[f64; 3]> = pts.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], [1.0, 2.0, 3.0]);
        assert_eq!(collected[1], [4.0, 5.0, 6.0]);
    }

    #[test]
    fn for_loop() {
        let pts = Points::from_vec(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let mut count = 0;
        for _p in &pts {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn exact_size() {
        let pts = Points::from_vec(vec![[0.0; 3]; 5]);
        assert_eq!(pts.iter().len(), 5);
    }

    #[test]
    fn bounds_computation() {
        let pts = Points::from_vec(vec![[0.0, -1.0, 2.0], [3.0, 4.0, -5.0]]);
        let bb = pts.bounds();
        assert_eq!(bb.x_min, 0.0);
        assert_eq!(bb.x_max, 3.0);
        assert_eq!(bb.y_min, -1.0);
        assert_eq!(bb.y_max, 4.0);
        assert_eq!(bb.z_min, -5.0);
        assert_eq!(bb.z_max, 2.0);
    }

    #[test]
    fn from_fn() {
        let pts = Points::from_fn(5, |i| [i as f64, 0.0, 0.0]);
        assert_eq!(pts.len(), 5);
        assert_eq!(pts.get(3), [3.0, 0.0, 0.0]);
    }

    #[test]
    fn to_vec() {
        let pts = Points::from_vec(vec![[1.0, 2.0, 3.0]]);
        let v = pts.to_vec();
        assert_eq!(v, vec![[1.0, 2.0, 3.0]]);
    }

    #[test]
    fn centroid() {
        let pts = Points::from_vec(vec![[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]]);
        let c = pts.centroid();
        assert!((c[0] - 1.0).abs() < 1e-10);
        assert!((c[1] - 2.0).abs() < 1e-10);
        assert!((c[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn transform() {
        let mut pts = Points::from_vec(vec![[1.0, 2.0, 3.0]]);
        pts.transform(|p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0]);
        assert_eq!(pts.get(0), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn flat_slice() {
        let pts = Points::from_vec(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let flat = pts.as_flat_slice();
        assert_eq!(flat, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn flat_slice_mut() {
        let mut pts = Points::from_vec(vec![[1.0, 0.0, 0.0]]);
        pts.as_flat_slice_mut()[0] = 99.0;
        assert_eq!(pts.get(0)[0], 99.0);
    }

    #[test]
    fn index_trait() {
        let pts = Points::from_vec(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        assert_eq!(pts[0], [1.0, 2.0, 3.0]);
        assert_eq!(pts[1], [4.0, 5.0, 6.0]);
    }

    #[test]
    fn from_iterator() {
        let pts: Points<f64> = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]].into_iter().collect();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts.get(0), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn extend_trait() {
        let mut pts = Points::from_vec(vec![[0.0, 0.0, 0.0]]);
        pts.extend(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(pts.len(), 3);
    }
}
