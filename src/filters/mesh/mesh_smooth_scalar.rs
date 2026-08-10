//! Smooth a scalar field on mesh vertices using Laplacian averaging.

/// Smooth a scalar field on mesh vertices using Laplacian averaging.
///
/// Single implementation lives in [`crate::filters::mesh::mesh_scalar_smooth`];
/// the smoothed values replace the array of the same name in the output.
pub use crate::filters::mesh::mesh_scalar_smooth::smooth_scalar;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};
    #[test]
    fn test_smooth_scalar() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 100.0, 0.0, 0.0],
                1,
            )));
        let r = smooth_scalar(&mesh, "v", 5, 0.5);
        let arr = r.point_data().get_array("v").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b);
        assert!(b[0] < 100.0); // smoothed toward neighbors
    }
}
