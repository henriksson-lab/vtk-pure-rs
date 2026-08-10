//! Estimate principal curvatures and directions at each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Principal curvatures with the derived mean/Gaussian curvature pair.
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::curvature_tensor::principal_curvatures`] (which
/// follows `vtkCurvatures::GetMaximumCurvature`/`GetMinimumCurvature`); this
/// entry point additionally publishes the "MeanCurv"/"GaussCurv" arrays it has
/// always produced.
pub fn principal_curvatures(mesh: &PolyData) -> PolyData {
    let mut r = crate::filters::mesh::curvature_tensor::principal_curvatures(mesh);
    let n = mesh.points.len();
    let fetch = |pd: &PolyData, name: &str| -> Vec<f64> {
        pd.point_data()
            .get_array(name)
            .map(|a| a.to_f64_vec())
            .unwrap_or_else(|| vec![0.0; n])
    };
    let k1 = fetch(&r, "K1");
    let k2 = fetch(&r, "K2");

    let len = k1.len().min(k2.len());
    let mean: Vec<f64> = (0..len).map(|i| (k1[i] + k2[i]) / 2.0).collect();
    let gaussian: Vec<f64> = (0..len).map(|i| k1[i] * k2[i]).collect();

    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("MeanCurv", mean, 1)));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussCurv",
            gaussian,
            1,
        )));
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = principal_curvatures(&m);
        assert!(r.point_data().get_array("K1").is_some());
        assert!(r.point_data().get_array("K2").is_some());
        assert!(r.point_data().get_array("MeanCurv").is_some());
        assert!(r.point_data().get_array("GaussCurv").is_some());
    }

    #[test]
    fn mean_and_gauss_follow_k1_k2() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = principal_curvatures(&m);
        let k1 = r.point_data().get_array("K1").unwrap().to_f64_vec();
        let k2 = r.point_data().get_array("K2").unwrap().to_f64_vec();
        let mean = r.point_data().get_array("MeanCurv").unwrap().to_f64_vec();
        let gauss = r.point_data().get_array("GaussCurv").unwrap().to_f64_vec();
        for i in 0..k1.len() {
            assert!((mean[i] - (k1[i] + k2[i]) / 2.0).abs() < 1e-12);
            assert!((gauss[i] - k1[i] * k2[i]).abs() < 1e-12);
        }
    }
}
