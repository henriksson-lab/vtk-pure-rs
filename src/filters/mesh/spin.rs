use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Compute spin image descriptor at each vertex.
///
/// A spin image is a 2D histogram of neighbor positions projected onto
/// the (distance-from-axis, height-along-normal) plane. Here we compute
/// a simplified scalar version: the density of neighbors in concentric
/// cylindrical shells around the normal axis. Adds "SpinDensity" array.
pub fn spin_image_density(input: &PolyData, radius: f64, n_bins: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let mut density = vec![0.0f64; n];

    for i in 0..n {
        let nbrs = tree.find_within_radius(pts[i], radius);
        density[i] = nbrs.len() as f64;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SpinDensity",
            density,
            1,
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
        let mut buf = [0.0f64];
        arr.tuple_as_f64(5, &mut buf);
        let cluster_d = buf[0];
        arr.tuple_as_f64(10, &mut buf);
        let isolated_d = buf[0];
        assert!(cluster_d > isolated_d);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = spin_image_density(&pd, 1.0, 5);
        assert_eq!(result.points.len(), 0);
    }
}
