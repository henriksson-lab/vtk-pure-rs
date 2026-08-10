use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Build the adjacency matrix of a mesh as sparse (row, col, weight) triples.
///
/// Weight = Euclidean edge length. Useful for graph algorithms.
pub fn adjacency_matrix(input: &PolyData) -> Vec<(usize, usize, f64)> {
    let mut edges: HashMap<(usize, usize), f64> = HashMap::new();
    let n = input.points.len();
    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let a_id = cell[i];
            let b_id = cell[(i + 1) % cell.len()];
            if a_id < 0 || b_id < 0 {
                continue;
            }
            let a = a_id as usize;
            let b = b_id as usize;
            if a >= n || b >= n || a == b {
                continue;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            edges.entry(key).or_insert_with(|| {
                let pa = input.points.get(a);
                let pb = input.points.get(b);
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
            });
        }
    }
    let mut result: Vec<(usize, usize, f64)> =
        edges.into_iter().map(|((a, b), w)| (a, b, w)).collect();
    result.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    result
}

/// Compute betweenness centrality (approximate) for each vertex.
///
/// This is Brandes' algorithm — the same algorithm Boost (and therefore
/// `vtkBoostBrandesCentrality`) uses — restricted to `num_sources` evenly
/// spaced source vertices. Adds an unnormalized "Centrality" scalar, matching
/// Boost's `brandes_betweenness_centrality`. Higher values = more shortest
/// paths pass through that vertex.
pub fn betweenness_centrality(input: &PolyData, num_sources: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        let mut pd = input.clone();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Centrality",
                Vec::new(),
                1,
            )));
        return pd;
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    add_adjacency(&mut adj, n, input.polys.iter(), true);
    add_adjacency(&mut adj, n, input.lines.iter(), false);

    let mut centrality = vec![0.0f64; n];
    let sources = num_sources.min(n);

    // Use evenly spaced source vertices
    for si in 0..sources {
        let src = si * n / sources;
        // BFS from src, counting shortest paths
        let mut dist = vec![usize::MAX; n];
        let mut npaths = vec![0u64; n];
        let mut queue = std::collections::VecDeque::new();
        let mut order = Vec::new();

        dist[src] = 0;
        npaths[src] = 1;
        queue.push_back(src);
        while let Some(v) = queue.pop_front() {
            order.push(v);
            for &nb in &adj[v] {
                if dist[nb] == usize::MAX {
                    dist[nb] = dist[v] + 1;
                    queue.push_back(nb);
                }
                if dist[nb] == dist[v] + 1 {
                    npaths[nb] += npaths[v];
                }
            }
        }

        // Accumulate centrality
        let mut delta = vec![0.0f64; n];
        for &v in order.iter().rev() {
            for &nb in &adj[v] {
                if dist[nb] == dist[v] + 1 && npaths[nb] > 0 {
                    delta[v] += (npaths[v] as f64 / npaths[nb] as f64) * (1.0 + delta[nb]);
                }
            }
            if v != src {
                centrality[v] += delta[v];
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Centrality",
            centrality,
            1,
        )));
    pd
}

/// Add the edges of `cells` to `adj`. Polygonal cells are closed loops, line
/// cells are open polylines.
fn add_adjacency<'a, I>(adj: &mut [Vec<usize>], n: usize, cells: I, closed: bool)
where
    I: IntoIterator<Item = &'a [i64]>,
{
    for cell in cells {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let edge_count = if closed { nc } else { nc - 1 };
        for i in 0..edge_count {
            let a_id = cell[i];
            let b_id = cell[(i + 1) % nc];
            if a_id < 0 || b_id < 0 {
                continue;
            }
            let a = a_id as usize;
            let b = b_id as usize;
            if a >= n || b >= n || a == b {
                continue;
            }
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacency_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let adj = adjacency_matrix(&pd);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn centrality_line() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = betweenness_centrality(&pd, 3);
        let arr = result.point_data().get_array("Centrality").unwrap();
        let mut buf = [0.0f64];
        // Middle vertex should have higher centrality
        arr.tuple_as_f64(2, &mut buf);
        let mid = buf[0];
        arr.tuple_as_f64(0, &mut buf);
        let end = buf[0];
        assert!(mid >= end);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(adjacency_matrix(&pd).is_empty());
        let result = betweenness_centrality(&pd, 3);
        let arr = result.point_data().get_array("Centrality").unwrap();
        assert_eq!(arr.num_tuples(), 0);
    }

    #[test]
    fn centrality_excludes_path_endpoints() {
        // Line cells contribute edges; on the path 0-1-2 only the middle
        // vertex lies on a shortest path between two other vertices.
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);

        let result = betweenness_centrality(&mesh, 3);
        let arr = result.point_data().get_array("Centrality").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        // Brandes accumulates each unordered pair twice on an undirected graph.
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 2.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
