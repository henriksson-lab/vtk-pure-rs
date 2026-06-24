use crate::data::{CellArray, Points, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Find the shortest path between two points on a mesh along edges.
///
/// Uses Dijkstra's algorithm on the mesh edge graph. Returns a PolyData
/// containing the path as a polyline.
pub fn geodesic_path(input: &PolyData, start: usize, end: usize) -> PolyData {
    let n = input.points.len();
    if start >= n || end >= n {
        return PolyData::new();
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
            adj[a].push((b, d));
            adj[b].push((a, d));
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
    let mut prev = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    dist[start] = 0.0;
    heap.push(S(0.0, start));

    while let Some(S(d, u)) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        if u == end {
            break;
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                prev[v] = u;
                heap.push(S(nd, v));
            }
        }
    }

    if dist[end] == f64::MAX {
        return PolyData::new();
    }

    // Reconstruct path
    let mut path = vec![end];
    let mut cur = end;
    while cur != start && prev[cur] != usize::MAX {
        cur = prev[cur];
        path.push(cur);
    }
    path.reverse();

    let mut out_points = Points::<f64>::new();
    let ids: Vec<i64> = path
        .iter()
        .map(|&p| {
            let idx = out_points.len() as i64;
            out_points.push(input.points.get(p));
            idx
        })
        .collect();

    let mut out_lines = CellArray::new();
    out_lines.push_cell(&ids);

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

/// Compute the geodesic path length between two points.
pub fn geodesic_path_length(input: &PolyData, start: usize, end: usize) -> f64 {
    let path = geodesic_path(input, start, end);
    let mut total = 0.0;
    for cell in path.lines.iter() {
        for i in 0..cell.len() - 1 {
            let a = path.points.get(cell[i] as usize);
            let b = path.points.get(cell[i + 1] as usize);
            total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_on_line() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let path = geodesic_path(&pd, 0, 2);
        assert!(path.points.len() >= 2); // direct edge or via vertex 1
        assert_eq!(path.lines.num_cells(), 1);
    }

    #[test]
    fn path_length() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let len = geodesic_path_length(&pd, 0, 2);
        // Direct: 5.0, via edge: 3+4=7 or 5
        assert!(len > 0.0);
    }

    #[test]
    fn no_path() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        // No cells connecting them
        let path = geodesic_path(&pd, 0, 1);
        assert_eq!(path.lines.num_cells(), 0);
    }

    #[test]
    fn same_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let path = geodesic_path(&pd, 0, 0);
        assert_eq!(path.points.len(), 1);
    }
}
