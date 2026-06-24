use crate::data::{AnyDataArray, DataArray, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Geodesic Voronoi partition: assign each vertex to nearest seed via
/// edge-weighted Dijkstra (true geodesic, not Euclidean).
///
/// More accurate than `mesh_voronoi_partition` which uses Euclidean distance.
/// Adds "GeodesicRegion" scalar.
pub fn geodesic_voronoi(input: &PolyData, seed_indices: &[usize]) -> PolyData {
    let n = input.points.len();
    if n == 0 || seed_indices.is_empty() {
        return input.clone();
    }

    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            if !adj[a].iter().any(|&(v, _)| v == b) {
                adj[a].push((b, d));
                adj[b].push((a, d));
            }
        }
    }

    #[derive(PartialEq)]
    struct S(f64, usize, usize); // (dist, vertex, seed_id)
    impl Eq for S {}
    impl PartialOrd for S {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for S {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let mut dist = vec![f64::MAX; n];
    let mut region = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();

    for (si, &seed) in seed_indices.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            region[seed] = si;
            heap.push(S(0.0, seed, si));
        }
    }

    while let Some(S(d, u, sid)) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                region[v] = sid;
                heap.push(S(nd, v, sid));
            }
        }
    }

    let region_f: Vec<f64> = region
        .iter()
        .map(|&r| if r == usize::MAX { -1.0 } else { r as f64 })
        .collect();
    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicRegion",
            region_f,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_regions() {
        let mut pd = PolyData::new();
        for i in 0..7 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[4, 5, 6]);

        let result = geodesic_voronoi(&pd, &[0, 6]);
        let arr = result.point_data().get_array("GeodesicRegion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(6, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn single_seed() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = geodesic_voronoi(&pd, &[0]);
        let arr = result.point_data().get_array("GeodesicRegion").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = geodesic_voronoi(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
