//! Data-parallel execution framework using rayon.
//!
//! Provides utilities for parallel-over-points and parallel-over-cells
//! operations on PolyData and ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};
use rayon::prelude::*;

/// Apply a function to each point in parallel, producing a scalar array.
pub fn parallel_map_points(
    mesh: &PolyData,
    array_name: &str,
    f: impl Fn([f64; 3]) -> f64 + Send + Sync,
) -> PolyData {
    let n = mesh.points.len();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let values: Vec<f64> = positions.par_iter().map(|p| f(*p)).collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    result
}

/// Apply a function to each point in parallel, producing a 3-component vector array.
pub fn parallel_map_points_vec3(
    mesh: &PolyData,
    array_name: &str,
    f: impl Fn([f64; 3]) -> [f64; 3] + Send + Sync,
) -> PolyData {
    let n = mesh.points.len();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let values: Vec<f64> = positions
        .par_iter()
        .flat_map(|p| {
            let v = f(*p);
            vec![v[0], v[1], v[2]]
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 3,
        )));
    result
}

/// Apply a function to each voxel of an ImageData in parallel.
pub fn parallel_map_image(
    image: &ImageData,
    array_name: &str,
    f: impl Fn(f64, f64, f64) -> f64 + Send + Sync,
) -> ImageData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();
    let total = dims[0] * dims[1] * dims[2];

    let values: Vec<f64> = (0..total)
        .into_par_iter()
        .map(|idx| {
            let iz = idx / (dims[0] * dims[1]);
            let rem = idx % (dims[0] * dims[1]);
            let iy = rem / dims[0];
            let ix = rem % dims[0];
            let x = origin[0] + ix as f64 * spacing[0];
            let y = origin[1] + iy as f64 * spacing[1];
            let z = origin[2] + iz as f64 * spacing[2];
            f(x, y, z)
        })
        .collect();

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    result
}

/// Transform point positions in parallel.
pub fn parallel_transform_points(
    mesh: &PolyData,
    f: impl Fn([f64; 3]) -> [f64; 3] + Send + Sync,
) -> PolyData {
    let n = mesh.points.len();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let new_positions: Vec<[f64; 3]> = positions.par_iter().map(|p| f(*p)).collect();

    let mut result = mesh.clone();
    result.points = crate::data::Points::from(new_positions);
    result
}

/// Parallel reduce over points (e.g., sum, min, max).
pub fn parallel_reduce_points<T: Send>(
    mesh: &PolyData,
    identity: T,
    f: impl Fn([f64; 3]) -> T + Send + Sync,
    reduce: impl Fn(T, T) -> T + Send + Sync,
) -> T
where
    T: Clone + Sync,
{
    let n = mesh.points.len();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    positions
        .par_iter()
        .map(|p| f(*p))
        .reduce(|| identity.clone(), |a, b| reduce(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_points_scalar() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let result = parallel_map_points(&mesh, "dist", |p| {
            (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
        });
        let arr = result.point_data().get_array("dist").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn map_image() {
        let img = ImageData::with_dimensions(5, 5, 5).with_spacing([1.0, 1.0, 1.0]);
        let result = parallel_map_image(&img, "field", |x, y, z| x + y + z);
        assert!(result.point_data().get_array("field").is_some());
    }

    #[test]
    fn transform_points() {
        let mesh = PolyData::from_points(vec![[1.0, 2.0, 3.0]]);
        let result = parallel_transform_points(&mesh, |p| [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0]);
        let p = result.points.get(0);
        assert!((p[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn reduce_sum() {
        let mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        let sum = parallel_reduce_points(&mesh, 0.0, |p| p[0], |a, b| a + b);
        assert!((sum - 6.0).abs() < 1e-10);
    }
}
