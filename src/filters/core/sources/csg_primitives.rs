//! CSG-style primitive shapes with implicit function evaluation.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Evaluate sphere SDF on a grid.
pub fn sphere_sdf(
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    center: [f64; 3],
    radius: f64,
) -> ImageData {
    implicit_field(dims, spacing, origin, |x, y, z| {
        ((x - center[0]).powi(2) + (y - center[1]).powi(2) + (z - center[2]).powi(2)).sqrt()
            - radius
    })
}

/// Evaluate box SDF on a grid.
pub fn box_sdf(
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    center: [f64; 3],
    half_size: [f64; 3],
) -> ImageData {
    implicit_field(dims, spacing, origin, |x, y, z| {
        let dx = (x - center[0]).abs() - half_size[0];
        let dy = (y - center[1]).abs() - half_size[1];
        let dz = (z - center[2]).abs() - half_size[2];
        let outside = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2) + dz.max(0.0).powi(2)).sqrt();
        let inside = dx.max(dy).max(dz).min(0.0);
        outside + inside
    })
}

/// CSG union of two SDF fields (min).
pub fn sdf_union(a: &ImageData, b: &ImageData) -> ImageData {
    sdf_binary_op(a, b, f64::min)
}

/// CSG intersection of two SDF fields (max).
pub fn sdf_intersection(a: &ImageData, b: &ImageData) -> ImageData {
    sdf_binary_op(a, b, f64::max)
}

/// CSG difference (a minus b).
pub fn sdf_difference(a: &ImageData, b: &ImageData) -> ImageData {
    sdf_binary_op(a, b, |x, y| x.max(-y))
}

fn sdf_binary_op(a: &ImageData, b: &ImageData, op: impl Fn(f64, f64) -> f64) -> ImageData {
    let arr_a = a.point_data().get_array("SDF").unwrap();
    let arr_b = b.point_data().get_array("SDF").unwrap();
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ba);
            arr_b.tuple_as_f64(i, &mut bb);
            op(ba[0], bb[0])
        })
        .collect();
    let dims = a.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(a.spacing())
        .with_origin(a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("SDF", data, 1)))
}

fn implicit_field(
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    f: impl Fn(f64, f64, f64) -> f64,
) -> ImageData {
    let n = dims[0] * dims[1] * dims[2];
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (dims[0] * dims[1]);
            let rem = idx % (dims[0] * dims[1]);
            let iy = rem / dims[0];
            let ix = rem % dims[0];
            f(
                origin[0] + ix as f64 * spacing[0],
                origin[1] + iy as f64 * spacing[1],
                origin[2] + iz as f64 * spacing[2],
            )
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("SDF", data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sphere() {
        let s = sphere_sdf(
            [10, 10, 10],
            [0.2, 0.2, 0.2],
            [-1.0, -1.0, -1.0],
            [0.0, 0.0, 0.0],
            0.5,
        );
        let arr = s.point_data().get_array("SDF").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 10 + 5 * 100, &mut buf); // near center
        assert!(buf[0] < 0.0); // inside
    }
    #[test]
    fn test_csg() {
        let a = sphere_sdf(
            [8, 8, 8],
            [0.5, 0.5, 0.5],
            [-2.0, -2.0, -2.0],
            [0.0, 0.0, 0.0],
            1.0,
        );
        let b = box_sdf(
            [8, 8, 8],
            [0.5, 0.5, 0.5],
            [-2.0, -2.0, -2.0],
            [0.0, 0.0, 0.0],
            [0.8, 0.8, 0.8],
        );
        let u = sdf_union(&a, &b);
        let i = sdf_intersection(&a, &b);
        let d = sdf_difference(&a, &b);
        assert_eq!(u.dimensions(), [8, 8, 8]);
        assert_eq!(i.dimensions(), [8, 8, 8]);
        assert_eq!(d.dimensions(), [8, 8, 8]);
    }
}
