//! Graph algorithms on mesh edge connectivity: shortest paths, centrality, spanning tree.

use crate::data::{CellArray, Points, PolyData};

/// Compute shortest path between two vertices via Dijkstra.
///
/// Returns vertex indices along the path.
pub fn shortest_path(mesh: &PolyData, start: usize, end: usize) -> Vec<usize> {
    let n = mesh.points.len();
    if start >= n || end >= n {
        return Vec::new();
    }
    let adj = build_adj(mesh, n);
    let mut dist = vec![f64::MAX; n];
    let mut prev = vec![usize::MAX; n];
    dist[start] = 0.0;

    let mut heap = std::collections::BinaryHeap::new();
    heap.push(std::cmp::Reverse((OrdF64(0.0), start)));

    while let Some(std::cmp::Reverse((OrdF64(d), v))) = heap.pop() {
        if d > dist[v] {
            continue;
        }
        if v == end {
            break;
        }
        for &nb in &adj[v] {
            let pv = mesh.points.get(v);
            let pn = mesh.points.get(nb);
            let edge_len =
                ((pv[0] - pn[0]).powi(2) + (pv[1] - pn[1]).powi(2) + (pv[2] - pn[2]).powi(2))
                    .sqrt();
            let new_d = d + edge_len;
            if new_d < dist[nb] {
                dist[nb] = new_d;
                prev[nb] = v;
                heap.push(std::cmp::Reverse((OrdF64(new_d), nb)));
            }
        }
    }

    let mut path = Vec::new();
    let mut cur = end;
    while cur != usize::MAX {
        path.push(cur);
        if cur == start {
            break;
        }
        cur = prev[cur];
    }
    path.reverse();
    if path.first() == Some(&start) {
        path
    } else {
        Vec::new()
    }
}

/// Extract shortest path as a PolyData polyline.
pub fn shortest_path_polyline(mesh: &PolyData, start: usize, end: usize) -> PolyData {
    let path = shortest_path(mesh, start, end);
    if path.is_empty() {
        return PolyData::new();
    }
    let mut pts = Points::<f64>::new();
    let ids: Vec<i64> = path
        .iter()
        .enumerate()
        .map(|(i, &vi)| {
            pts.push(mesh.points.get(vi));
            i as i64
        })
        .collect();
    let mut lines = CellArray::new();
    lines.push_cell(&ids);
    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Compute betweenness centrality for each vertex (approximate via sampling).
///
/// Re-exported from [`crate::filters::mesh::abstract_graph`], which holds the
/// single implementation (Brandes' algorithm, as used by
/// `vtkBoostBrandesCentrality`).
pub use crate::filters::mesh::abstract_graph::betweenness_centrality;

/// Compute minimum spanning tree of the mesh edge graph.
///
/// Re-exported from [`crate::filters::mesh::minimum_spanning_tree`], which
/// holds the single Kruskal implementation.
pub use crate::filters::mesh::minimum_spanning_tree::minimum_spanning_tree;

#[derive(Clone, Copy, PartialEq)]
struct OrdF64(f64);
impl Eq for OrdF64 {}
impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&o.0)
    }
}
impl Ord for OrdF64 {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.partial_cmp(o).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    add_adj_cells(mesh.polys.iter(), true, n, &mut adj);
    add_adj_cells(mesh.lines.iter(), false, n, &mut adj);
    adj.into_iter()
        .map(|s| {
            let mut neighbors: Vec<usize> = s.into_iter().collect();
            neighbors.sort_unstable();
            neighbors
        })
        .collect()
}

fn add_adj_cells<'a, I>(
    cells: I,
    closed: bool,
    n: usize,
    adj: &mut [std::collections::HashSet<usize>],
) where
    I: IntoIterator<Item = &'a [i64]>,
{
    for cell in cells {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let edge_count = if closed { nc } else { nc - 1 };
        for i in 0..edge_count {
            if cell[i] < 0 || cell[(i + 1) % nc] < 0 {
                continue;
            }
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let path = shortest_path(&mesh, 0, 24);
        assert!(!path.is_empty());
        assert_eq!(path[0], 0);
        assert_eq!(*path.last().unwrap(), 24);
    }
    #[test]
    fn path_polyline() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let line = shortest_path_polyline(&mesh, 0, 2);
        assert!(line.lines.num_cells() >= 1);
    }
    #[test]
    fn path_on_line_cell() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);

        assert_eq!(shortest_path(&mesh, 0, 2), vec![0, 1, 2]);
    }
}
