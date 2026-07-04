//! Build explicit edge graph from mesh topology.
use crate::data::PolyData;
pub struct EdgeGraph {
    pub vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub weights: Vec<f64>,
}
pub fn build_edge_graph(mesh: &PolyData) -> EdgeGraph {
    let n = mesh.points.len();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut edges = Vec::new();
    let mut weights = Vec::new();

    for cell in mesh.lines.iter() {
        for pair in cell.windows(2) {
            insert_edge(
                mesh,
                n,
                pair[0],
                pair[1],
                &mut seen,
                &mut edges,
                &mut weights,
            );
        }
    }
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(
                mesh,
                n,
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut seen,
                &mut edges,
                &mut weights,
            );
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(mesh, n, tri[0], tri[1], &mut seen, &mut edges, &mut weights);
            insert_edge(mesh, n, tri[1], tri[2], &mut seen, &mut edges, &mut weights);
            insert_edge(mesh, n, tri[2], tri[0], &mut seen, &mut edges, &mut weights);
        }
    }
    EdgeGraph {
        vertices: n,
        edges,
        weights,
    }
}
fn insert_edge(
    mesh: &PolyData,
    n: usize,
    a: i64,
    b: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    edges: &mut Vec<(usize, usize)>,
    weights: &mut Vec<f64>,
) {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a >= n || b >= n || a == b {
        return;
    }
    let key = (a.min(b), a.max(b));
    if seen.insert(key) {
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let d =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        edges.push(key);
        weights.push(d);
    }
}
pub fn shortest_path_dijkstra(graph: &EdgeGraph, start: usize, end: usize) -> (Vec<usize>, f64) {
    let n = graph.vertices;
    if start >= n || end >= n {
        return (vec![], f64::INFINITY);
    }
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (i, &(a, b)) in graph.edges.iter().enumerate() {
        if a < n && b < n && i < graph.weights.len() {
            adj[a].push((b, graph.weights[i]));
            adj[b].push((a, graph.weights[i]));
        }
    }
    let mut dist = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    dist[start] = 0.0;
    for _ in 0..n {
        let u = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let u = match u {
            Some(u) => u,
            None => {
                break;
            }
        };
        if u == end {
            break;
        }
        visited[u] = true;
        for &(v, w) in &adj[u] {
            let alt = dist[u] + w;
            if alt < dist[v] {
                dist[v] = alt;
                prev[v] = u;
            }
        }
    }
    if dist[end].is_infinite() {
        return (vec![], f64::INFINITY);
    }
    let mut path = vec![end];
    let mut cur = end;
    while cur != start {
        cur = prev[cur];
        if cur == usize::MAX {
            return (vec![], f64::INFINITY);
        }
        path.push(cur);
    }
    path.reverse();
    (path, dist[end])
}
pub fn minimum_spanning_tree(graph: &EdgeGraph) -> Vec<(usize, usize, f64)> {
    let n = graph.vertices;
    let mut parent: Vec<usize> = (0..n).collect();
    let mut sorted: Vec<(usize, (usize, usize))> = (0..graph.edges.len())
        .map(|i| (i, graph.edges[i]))
        .collect();
    sorted.sort_by(|a, b| {
        graph.weights[a.0]
            .partial_cmp(&graph.weights[b.0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut mst = Vec::new();
    for (i, (a, b)) in sorted {
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        if ra != rb {
            parent[rb] = ra;
            mst.push((a, b, graph.weights[i]));
        }
    }
    mst
}
fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_graph() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let g = build_edge_graph(&m);
        assert_eq!(g.edges.len(), 5);
    }
    #[test]
    fn test_path() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let g = build_edge_graph(&m);
        let (path, d) = shortest_path_dijkstra(&g, 0, 3);
        assert!(path.len() >= 2);
        assert!(d > 0.0);
    }
    #[test]
    fn test_mst() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let g = build_edge_graph(&m);
        let mst = minimum_spanning_tree(&g);
        assert_eq!(mst.len(), 3);
    } // 4 vertices -> 3 MST edges
}
