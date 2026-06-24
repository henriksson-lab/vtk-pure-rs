use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::ImplicitFunction;

/// Sample an implicit function on an ImageData grid.
///
/// Evaluates `func(x, y, z)` at each grid point and stores the result
/// as a scalar point data array.
pub fn sample_implicit_function(
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    name: &str,
    func: &dyn ImplicitFunction,
) -> ImageData {
    let mut values = Vec::with_capacity(dims[0] * dims[1] * dims[2]);
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let x = origin[0] + i as f64 * spacing[0];
                let y = origin[1] + j as f64 * spacing[1];
                let z = origin[2] + k as f64 * spacing[2];
                values.push(func.evaluate(x, y, z));
            }
        }
    }

    let mut img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(origin);
    let arr = DataArray::from_vec(name, values, 1);
    img.point_data_mut().add_array(AnyDataArray::F64(arr));
    img.point_data_mut().set_active_scalars(name);
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ImplicitPlane, ImplicitSphere};

    #[test]
    fn sphere_distance_field() {
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let img = sample_implicit_function(
            [5, 5, 5],
            [0.5, 0.5, 0.5],
            [-1.0, -1.0, -1.0],
            "distance",
            &sphere,
        );
        assert_eq!(img.dimensions(), [5, 5, 5]);
        let scalars = img.point_data().scalars().unwrap();
        assert_eq!(scalars.num_tuples(), 125);
        // Center voxel (2,2,2) at origin should have negative value (inside sphere)
        let center_val = img.scalar_at(2, 2, 2).unwrap();
        assert!(
            center_val < 0.0,
            "center should be inside sphere, got {center_val}"
        );
    }

    #[test]
    fn plane_field() {
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let img = sample_implicit_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [-2.0, 0.0, 0.0],
            "plane",
            &plane,
        );
        // At x=-2: negative, at x=2: positive
        let v0 = img.scalar_at(0, 0, 0).unwrap();
        let v4 = img.scalar_at(4, 0, 0).unwrap();
        assert!(v0 < 0.0);
        assert!(v4 > 0.0);
    }
}
