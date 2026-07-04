//! Farthest point sampling on mesh using geodesic distance.
use crate::data::{CellArray, Points, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(PartialEq)]
struct State {
    cost: f64,
    node: usize,
}
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

pub fn farthest_point_sample(mesh: &PolyData, num_samples: usize, seed: usize) -> Vec<usize> {
    let n = mesh.points.len();
    if n == 0 || num_samples == 0 {
        return vec![];
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        if !valid_polygon_cell(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut nb);
        }
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(mesh, strip, &mut nb);
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut nb);
        }
    }
    let mut selected = vec![seed.min(n - 1)];
    let mut min_dist = vec![f64::INFINITY; n];
    let dist = dijkstra(&nb, selected[0]);
    for i in 0..n {
        min_dist[i] = dist[i];
    }
    for _ in 1..num_samples.min(n) {
        let farthest = (0..n).filter(|i| !selected.contains(i)).max_by(|&a, &b| {
            min_dist[a]
                .partial_cmp(&min_dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        match farthest {
            Some(f) => {
                selected.push(f);
                let dist = dijkstra(&nb, f);
                for i in 0..n {
                    min_dist[i] = min_dist[i].min(dist[i]);
                }
            }
            None => {
                break;
            }
        }
    }
    selected
}
pub fn farthest_point_sample_polydata(
    mesh: &PolyData,
    num_samples: usize,
    seed: usize,
) -> PolyData {
    let indices = farthest_point_sample(mesh, num_samples, seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for &i in &indices {
        let idx = pts.len();
        pts.push(mesh.points.get(i));
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

fn dijkstra(adj: &[Vec<(usize, f64)>], seed: usize) -> Vec<f64> {
    let n = adj.len();
    let mut dist = vec![f64::INFINITY; n];
    dist[seed] = 0.0;
    let mut heap = BinaryHeap::new();
    heap.push(State {
        cost: 0.0,
        node: seed,
    });
    while let Some(State { cost, node }) = heap.pop() {
        if cost > dist[node] {
            continue;
        }
        for &(next_node, weight) in &adj[node] {
            let next = cost + weight;
            if next < dist[next_node] {
                dist[next_node] = next;
                heap.push(State {
                    cost: next,
                    node: next_node,
                });
            }
        }
    }
    dist
}

fn add_triangle_strip_edges(mesh: &PolyData, strip: &[i64], adj: &mut [Vec<(usize, f64)>]) {
    for tri in strip.windows(3) {
        if !valid_triangle(tri, adj.len()) {
            continue;
        }
        add_edge(mesh, tri[0], tri[1], adj);
        add_edge(mesh, tri[1], tri[2], adj);
        add_edge(mesh, tri[2], tri[0], adj);
    }
}

fn add_edge(mesh: &PolyData, a: i64, b: i64, adj: &mut [Vec<(usize, f64)>]) {
    let n = adj.len();
    let Some(a) = valid_point_id(a, n) else {
        return;
    };
    let Some(b) = valid_point_id(b, n) else {
        return;
    };
    if a == b {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !adj[a].iter().any(|&(v, _)| v == b) {
        adj[a].push((b, d));
    }
    if !adj[b].iter().any(|&(v, _)| v == a) {
        adj[b].push((a, d));
    }
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn valid_polygon_cell(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [1.5, 3.0, 0.0],
                [3.0, 3.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let s = farthest_point_sample(&m, 3, 0);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], 0);
    }
    #[test]
    fn test_polydata() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = farthest_point_sample_polydata(&m, 2, 0);
        assert_eq!(r.points.len(), 2);
    }
}
