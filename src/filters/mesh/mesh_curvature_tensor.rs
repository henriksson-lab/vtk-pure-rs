//! Compute Gaussian and mean curvature using vtkCurvatures-style formulas.
use crate::data::PolyData;

pub fn curvature_tensor(mesh: &PolyData) -> PolyData {
    crate::filters::geometry::curvatures::curvatures(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_curvature_tensor() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = curvature_tensor(&mesh);
        assert!(r.point_data().get_array("Mean_Curvature").is_some());
        assert!(r.point_data().get_array("Gauss_Curvature").is_some());
    }
}
