//! Gradient of a scalar field on a triangle mesh.
//!
//! The implementation lives in [`crate::filters::mesh::point_data_gradient`].

pub use crate::filters::mesh::point_data_gradient::scalar_gradient_on_mesh;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};

    #[test]
    fn linear_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.0],
                1,
            )));

        let result = scalar_gradient_on_mesh(&pd, "f");
        assert!(result.point_data().get_array("ScalarGradient").is_some());
        let mag = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [0.0f64];
        mag.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5); // gradient should be ~1 in X direction
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = scalar_gradient_on_mesh(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
