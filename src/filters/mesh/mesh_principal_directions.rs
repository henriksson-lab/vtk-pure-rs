//! Estimate principal curvature magnitudes at each vertex.
use crate::data::PolyData;

pub fn principal_curvatures(mesh: &PolyData) -> PolyData {
    super::principal_curvatures::compute_principal_curvatures(mesh)
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
}
