//! Apply Gaussian weighting based on distance from a point.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn gaussian_weight(mesh: &PolyData, center: [f64; 3], sigma: f64) -> PolyData {
    let n = mesh.points.len();
    let s2 = 2.0 * sigma * sigma;
    let weights: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let d2 = (p[0] - center[0]).powi(2)
                + (p[1] - center[1]).powi(2)
                + (p[2] - center[2]).powi(2);
            (-d2 / s2).exp()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianWeight",
            weights,
            1,
        )));
    result.point_data_mut().set_active_scalars("GaussianWeight");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gaussian() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = gaussian_weight(&mesh, [0.0, 0.0, 0.0], 1.0);
        let arr = r.point_data().get_array("GaussianWeight").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-9); // center point = weight 1
    }
}
