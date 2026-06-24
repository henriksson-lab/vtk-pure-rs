use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Compute local point density for each point in a point cloud.
///
/// For each point, counts the number of neighbors within `radius`
/// and stores it as a "Density" scalar. Also computes a normalized
/// density by dividing by the sphere volume.
pub fn point_cloud_density(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts = input.points.to_vec();
    let tree = KdTree::build(&pts);

    let sphere_vol = (4.0 / 3.0) * std::f64::consts::PI * radius * radius * radius;
    let inv_vol = if sphere_vol > 1e-15 {
        1.0 / sphere_vol
    } else {
        0.0
    };
    let mut counts = Vec::with_capacity(n);
    let mut densities = Vec::with_capacity(n);

    for i in 0..n {
        let count = tree.find_within_radius(pts[i], radius).len();
        counts.push(count as f64);
        densities.push(count as f64 * inv_vol);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborCount",
            counts,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Density", densities, 1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_grid_density() {
        let mut pd = PolyData::new();
        for i in 0..3 {
            for j in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }

        let result = point_cloud_density(&pd, 1.5);
        let arr = result.point_data().get_array("NeighborCount").unwrap();
        let mut buf = [0.0f64];
        // Center point (1,1) should have most neighbors
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] >= 5.0); // self + 4 cardinal neighbors at least
    }

    #[test]
    fn has_density_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = point_cloud_density(&pd, 2.0);
        assert!(result.point_data().get_array("Density").is_some());
        assert!(result.point_data().get_array("NeighborCount").is_some());
    }

    #[test]
    fn isolated_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);

        let result = point_cloud_density(&pd, 1.0);
        let arr = result.point_data().get_array("NeighborCount").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // only self
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_cloud_density(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
