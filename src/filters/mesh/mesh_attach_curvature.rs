//! Attach discrete curvature estimates as point data.
use crate::data::PolyData;
use crate::filters::mesh::curvature_simple;

pub fn attach_mean_curvature(mesh: &PolyData) -> PolyData {
    curvature_simple::mean_curvature(mesh)
}
pub fn attach_gaussian_curvature(mesh: &PolyData) -> PolyData {
    curvature_simple::gaussian_curvature(mesh)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_mean_curvature(&m);
        assert!(r.point_data().get_array("MeanCurvature").is_some());
    }
    #[test]
    fn test_gauss() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_gaussian_curvature(&m);
        assert!(r.point_data().get_array("GaussianCurvature").is_some());
    }
}
