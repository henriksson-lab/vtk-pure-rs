/// Estimate normals for a point cloud using PCA of local neighborhoods.
///
/// For each point, finds `k_neighbors` nearest neighbors (by Euclidean
/// distance), computes the 3x3 covariance matrix of the neighborhood,
/// and takes the eigenvector corresponding to the smallest eigenvalue
/// as the estimated normal direction. The normals are added as a
/// 3-component "Normals" point data array.
///
/// Re-exported from [`crate::filters::mesh::point_cloud_normals`], which holds
/// the single implementation (the faithful `vtkPCANormalEstimation` translation).
pub use crate::filters::mesh::point_cloud_normals::estimate_point_cloud_normals;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn planar_points_normal_is_z() {
        // Points in the XY plane should have normals along Z
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]);

        let result = estimate_point_cloud_normals(&pd, 5);
        let normals = result.point_data().get_array("Normals").unwrap();
        assert_eq!(normals.num_components(), 3);
        assert_eq!(normals.num_tuples(), 5);

        let mut buf = [0.0f64; 3];
        for i in 0..5 {
            normals.tuple_as_f64(i, &mut buf);
            // Normal should be (0,0,+/-1)
            assert!(buf[2].abs() > 0.99, "normal z={} at point {}", buf[2], i);
            assert!(buf[0].abs() < 0.01, "normal x={} at point {}", buf[0], i);
            assert!(buf[1].abs() < 0.01, "normal y={} at point {}", buf[1], i);
        }
    }

    #[test]
    fn empty_point_cloud() {
        let pd = PolyData::new();
        let result = estimate_point_cloud_normals(&pd, 5);
        assert!(result.point_data().get_array("Normals").is_none());
    }

    #[test]
    fn single_point_returns_some_normal() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = estimate_point_cloud_normals(&pd, 1);
        let normals = result.point_data().get_array("Normals").unwrap();
        assert_eq!(normals.num_tuples(), 1);
        // With a single point the covariance is zero, so we just check it doesn't crash
        let mut buf = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut buf);
        let len: f64 = (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt();
        // len could be 0 or 1, either is acceptable
        assert!(len < 1.01);
    }
}
