use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Assign each vertex to its nearest seed point (Voronoi partition on mesh).
///
/// Given a set of seed indices, partitions vertices by nearest seed
/// using Euclidean distance. Adds "VoronoiRegion" scalar.
pub fn mesh_voronoi_partition(input: &PolyData, seed_indices: &[usize]) -> PolyData {
    let n = input.points.len();
    if n == 0 || seed_indices.is_empty() {
        return input.clone();
    }

    let seed_pts: Vec<[f64; 3]> = seed_indices
        .iter()
        .filter(|&&i| i < n)
        .map(|&i| input.points.get(i))
        .collect();
    if seed_pts.is_empty() {
        return input.clone();
    }

    let tree = KdTree::build(&seed_pts);

    let mut region_ids = Vec::with_capacity(n);
    for i in 0..n {
        let p = input.points.get(i);
        let id = match tree.nearest(p) {
            Some((idx, _)) => idx as f64,
            None => -1.0,
        };
        region_ids.push(id);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiRegion",
            region_ids,
            1,
        )));
    pd
}

/// Compute the centroidal Voronoi tessellation (CVT) by iterating.
///
/// Alternately assigns vertices to nearest seed and moves seeds to
/// region centroids. Returns updated seed positions after `iterations`.
pub fn cvt_iterate(
    input: &PolyData,
    initial_seeds: &[[f64; 3]],
    iterations: usize,
) -> Vec<[f64; 3]> {
    let n = input.points.len();
    if n == 0 || initial_seeds.is_empty() {
        return initial_seeds.to_vec();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut seeds = initial_seeds.to_vec();
    let k = seeds.len();

    for _ in 0..iterations {
        let tree = KdTree::build(&seeds);

        // Assign each point to nearest seed
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];

        for &p in &pts {
            if let Some((idx, _)) = tree.nearest(p) {
                sums[idx][0] += p[0];
                sums[idx][1] += p[1];
                sums[idx][2] += p[2];
                counts[idx] += 1;
            }
        }

        // Move seeds to centroids
        for i in 0..k {
            if counts[i] > 0 {
                let c = counts[i] as f64;
                seeds[i] = [sums[i][0] / c, sums[i][1] / c, sums[i][2] / c];
            }
        }
    }

    seeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_basic() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let result = mesh_voronoi_partition(&pd, &[0, 9]);
        let arr = result.point_data().get_array("VoronoiRegion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // near seed 0
        arr.tuple_as_f64(9, &mut buf);
        assert_eq!(buf[0], 1.0); // near seed 1
    }

    #[test]
    fn cvt_converges() {
        let mut pd = PolyData::new();
        for i in 0..20 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let seeds = cvt_iterate(&pd, &[[0.0, 0.0, 0.0], [19.0, 0.0, 0.0]], 10);
        // Seeds should move toward region centroids
        assert!(seeds[0][0] > 0.0);
        assert!(seeds[1][0] < 19.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = mesh_voronoi_partition(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
