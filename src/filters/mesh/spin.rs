use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Compute spin image descriptor at each vertex.
///
/// A spin image is a 2D histogram of neighbor positions projected onto
/// the (distance-from-axis, height-along-normal) plane. Here we compute
/// a simplified radial version: the density of neighbors in concentric
/// spherical shells. Adds "SpinDensity" multi-component array.
pub fn spin_image_density(input: &PolyData, radius: f64, n_bins: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let n_bins = n_bins.max(1);
    if radius <= 0.0 {
        let mut pd = input.clone();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "SpinDensity",
                vec![0.0; n * n_bins],
                n_bins,
            )));
        return pd;
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut density = vec![0.0f64; n * n_bins];
    let bin_width = radius / n_bins as f64;

    for i in 0..n {
        let nbrs = tree.find_within_radius(pts[i], radius);
        for (neighbor_id, dist_sq) in nbrs {
            if neighbor_id == i {
                continue;
            }
            let distance = dist_sq.sqrt();
            let bin = ((distance / bin_width).floor() as usize).min(n_bins - 1);
            density[i * n_bins + bin] += 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SpinDensity",
            density,
            n_bins,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_varies() {
        let mut pd = PolyData::new();
        // Dense cluster
        for i in 0..10 {
            pd.points.push([i as f64 * 0.1, 0.0, 0.0]);
        }
        // Isolated point
        pd.points.push([100.0, 0.0, 0.0]);

        let result = spin_image_density(&pd, 1.0, 5);
        let arr = result.point_data().get_array("SpinDensity").unwrap();
        assert_eq!(arr.num_components(), 5);
        let mut buf = [0.0f64; 5];
        arr.tuple_as_f64(5, &mut buf);
        let cluster_d: f64 = buf.iter().sum();
        arr.tuple_as_f64(10, &mut buf);
        let isolated_d: f64 = buf.iter().sum();
        assert!(cluster_d > isolated_d);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = spin_image_density(&pd, 1.0, 5);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn non_positive_radius_still_adds_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);

        let result = spin_image_density(&pd, 0.0, 0);
        let arr = result.point_data().get_array("SpinDensity").unwrap();
        assert_eq!(arr.num_components(), 1);
        let mut buf = [1.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
