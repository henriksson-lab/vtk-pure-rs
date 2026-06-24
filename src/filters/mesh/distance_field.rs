use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute geodesic-like distance from a set of seed points along mesh edges.
///
/// Uses Dijkstra's algorithm on the mesh edge graph. `seed_indices` are
/// the starting points (distance = 0). Adds a "GeodesicDistance" scalar.
pub fn geodesic_distance(input: &PolyData, seed_indices: &[usize]) -> PolyData {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Build adjacency with edge lengths
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            adj[a].push((b, d));
            adj[b].push((a, d));
        }
    }

    #[derive(PartialEq)]
    struct State(f64, usize);
    impl Eq for State {}
    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let mut dist = vec![f64::MAX; n];
    let mut heap = BinaryHeap::new();

    for &s in seed_indices {
        if s < n {
            dist[s] = 0.0;
            heap.push(State(0.0, s));
        }
    }

    while let Some(State(d, u)) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(State(nd, v));
            }
        }
    }

    // Replace MAX with -1 for unreachable
    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_from_corner() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = geodesic_distance(&pd, &[0]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn multiple_seeds() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([2.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = geodesic_distance(&pd, &[0, 1]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn empty_seeds() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = geodesic_distance(&pd, &[]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], -1.0); // unreachable
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = geodesic_distance(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
