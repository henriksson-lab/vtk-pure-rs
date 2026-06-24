//! Graph algorithms on mesh edge connectivity: shortest paths, centrality, spanning tree.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Compute shortest path between two vertices via Dijkstra.
///
/// Returns vertex indices along the path.
pub fn shortest_path(mesh: &PolyData, start: usize, end: usize) -> Vec<usize> {
    let n = mesh.points.len();
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
pub fn betweenness_centrality(mesh: &PolyData, n_samples: usize) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut centrality = vec![0.0f64; n];

    let sample_step = (n / n_samples.max(1)).max(1);
    for src in (0..n).step_by(sample_step) {
        let (dists, prevs) = dijkstra(mesh, &adj, src);
        // Count how many shortest paths pass through each vertex
        for tgt in (0..n).step_by(sample_step) {
            if tgt == src {
                continue;
            }
            let mut cur = tgt;
            while cur != usize::MAX && cur != src {
                centrality[cur] += 1.0;
                cur = prevs[cur];
            }
        }
    }

    let max_c = centrality.iter().cloned().fold(0.0f64, f64::max).max(1.0);
    for c in &mut centrality {
        *c /= max_c;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Centrality",
            centrality,
            1,
        )));
    result
}

/// Compute minimum spanning tree of the mesh edge graph.
pub fn minimum_spanning_tree(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    // Collect unique edges with lengths
    let mut edges: Vec<(f64, usize, usize)> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let edge = (a.min(b), a.max(b));
            if seen.insert(edge) {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                edges.push((d, a, b));
            }
        }
    }
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Kruskal's algorithm
    let mut parent: Vec<usize> = (0..n).collect();
    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    };

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (_, a, b) in &edges {
        let ra = find(&mut parent, *a);
        let rb = find(&mut parent, *b);
        if ra != rb {
            parent[ra] = rb;
            let ia = *pt_map.entry(*a).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(*a));
                i
            });
            let ib = *pt_map.entry(*b).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(*b));
                i
            });
            lines.push_cell(&[ia as i64, ib as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

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
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn dijkstra(mesh: &PolyData, adj: &[Vec<usize>], src: usize) -> (Vec<f64>, Vec<usize>) {
    let n = adj.len();
    let mut dist = vec![f64::MAX; n];
    let mut prev = vec![usize::MAX; n];
    dist[src] = 0.0;
    let mut heap = std::collections::BinaryHeap::new();
    heap.push(std::cmp::Reverse((OrdF64(0.0), src)));
    while let Some(std::cmp::Reverse((OrdF64(d), v))) = heap.pop() {
        if d > dist[v] {
            continue;
        }
        for &nb in &adj[v] {
            let pv = mesh.points.get(v);
            let pn = mesh.points.get(nb);
            let el = ((pv[0] - pn[0]).powi(2) + (pv[1] - pn[1]).powi(2) + (pv[2] - pn[2]).powi(2))
                .sqrt();
            let nd = d + el;
            if nd < dist[nb] {
                dist[nb] = nd;
                prev[nb] = v;
                heap.push(std::cmp::Reverse((OrdF64(nd), nb)));
            }
        }
    }
    (dist, prev)
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
    fn mst() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..2 {
            for x in 0..2 {
                let bl = y * 3 + x;
                tris.push([bl, bl + 1, bl + 4]);
                tris.push([bl, bl + 4, bl + 3]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let tree = minimum_spanning_tree(&mesh);
        assert_eq!(tree.lines.num_cells(), 8); // 9 vertices - 1 = 8 MST edges
    }
    #[test]
    fn centrality() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..3 {
            for x in 0..3 {
                let bl = y * 4 + x;
                tris.push([bl, bl + 1, bl + 5]);
                tris.push([bl, bl + 5, bl + 4]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = betweenness_centrality(&mesh, 8);
        assert!(result.point_data().get_array("Centrality").is_some());
    }
}
