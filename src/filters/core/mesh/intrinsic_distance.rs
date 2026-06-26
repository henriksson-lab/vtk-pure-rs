use crate::data::{AnyDataArray, DataArray, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Compute approximate intrinsic (geodesic) distances from a set of source vertices.
///
/// Uses Dijkstra on the edge graph weighted by Euclidean edge lengths.
/// More accurate than hop-count `boundary_distance`. Adds "IntrinsicDistance".
pub fn intrinsic_distance(input: &PolyData, sources: &[usize]) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            add_edge(input, &mut adj, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(input, &mut adj, n, edge[0], edge[1]);
        }
    }

    #[derive(PartialEq)]
    struct S(f64, usize);
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
    let mut heap = BinaryHeap::new();
    for &s in sources {
        if s < n {
            dist[s] = 0.0;
            heap.push(S(0.0, s));
        }
    }

    while let Some(S(d, u)) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(S(nd, v));
            }
        }
    }

    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IntrinsicDistance",
            dist,
            1,
        )));
    pd.point_data_mut().set_active_scalars("IntrinsicDistance");
    pd
}

fn add_edge(input: &PolyData, adj: &mut [Vec<(usize, f64)>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }

    let pa = input.points.get(a);
    let pb = input.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
        .sqrt();
    if !adj[a].iter().any(|&(v, _)| v == b) {
        adj[a].push((b, d));
    }
    if !adj[b].iter().any(|&(v, _)| v == a) {
        adj[b].push((a, d));
    }
}

/// Compute geodesic distance between two specific vertices.
pub fn geodesic_distance_between(input: &PolyData, a: usize, b: usize) -> f64 {
    let result = intrinsic_distance(input, &[a]);
    if let Some(arr) = result.point_data().get_array("IntrinsicDistance") {
        if b < arr.num_tuples() {
            let mut buf = [0.0f64];
            arr.tuple_as_f64(b, &mut buf);
            return buf[0];
        }
    }
    -1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_distance() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 2, 3]);

        let result = intrinsic_distance(&pd, &[0]);
        let arr = result.point_data().get_array("IntrinsicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn between_two() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.polys.push_cell(&[0, 1]);

        let d = geodesic_distance_between(&pd, 0, 1);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn line_cells_are_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let d = geodesic_distance_between(&pd, 0, 1);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn disconnected() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        // No cells connecting them

        let result = intrinsic_distance(&pd, &[0]);
        let arr = result.point_data().get_array("IntrinsicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], -1.0); // unreachable
    }

    #[test]
    fn invalid_connectivity_is_ignored() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 99, 1]);

        let result = intrinsic_distance(&pd, &[0]);
        let arr = result.point_data().get_array("IntrinsicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], -1.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = intrinsic_distance(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
