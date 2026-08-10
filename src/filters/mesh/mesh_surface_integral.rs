//! Integrate a scalar field over the mesh surface.
//!
//! Re-exports the single implementation; see
//! [`crate::filters::mesh::surface_integral`].
pub use crate::filters::mesh::surface_integral::surface_integral;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};

    #[test]
    fn test_integral_quad_uses_full_polygon() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![2.0, 2.0, 2.0, 2.0],
                1,
            )));
        let result = surface_integral(&mesh, "f");
        assert!((result - 2.0).abs() < 1e-9);
    }
}
