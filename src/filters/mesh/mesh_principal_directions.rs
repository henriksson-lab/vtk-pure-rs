//! Estimate principal curvature magnitudes at each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Principal curvature magnitudes, published as "PrincipalCurvature1"/"PrincipalCurvature2".
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::curvature_tensor::principal_curvatures`], which
/// follows `vtkCurvatures::GetMaximumCurvature`/`GetMinimumCurvature`
/// (`k_max = H + sqrt(H^2 - K)`, `k_min = H - sqrt(H^2 - K)`). The VTK-named
/// `Maximum_Curvature`/`Minimum_Curvature` arrays are kept alongside the
/// aliases this entry point has always produced.
pub fn principal_curvatures(mesh: &PolyData) -> PolyData {
    let mut r = crate::filters::mesh::curvature_tensor::principal_curvatures(mesh);
    for (src, alias) in [
        ("Maximum_Curvature", "PrincipalCurvature1"),
        ("Minimum_Curvature", "PrincipalCurvature2"),
    ] {
        let values = match r.point_data().get_array(src) {
            Some(a) => a.to_f64_vec(),
            None => continue,
        };
        r.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(alias, values, 1)));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_principal() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = principal_curvatures(&mesh);
        assert!(r.point_data().get_array("PrincipalCurvature1").is_some());
        assert!(r.point_data().get_array("PrincipalCurvature2").is_some());
    }

    #[test]
    fn aliases_match_vtk_named_arrays() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = principal_curvatures(&mesh);
        let k_max = r
            .point_data()
            .get_array("Maximum_Curvature")
            .unwrap()
            .to_f64_vec();
        let alias = r
            .point_data()
            .get_array("PrincipalCurvature1")
            .unwrap()
            .to_f64_vec();
        assert_eq!(k_max, alias);
    }
}
