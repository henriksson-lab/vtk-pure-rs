//! Find the geodesic centroid (vertex minimizing max geodesic distance).
use crate::data::{AnyDataArray, DataArray, PolyData};
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
        other.cost.partial_cmp(&self.cost)
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub fn geodesic_centroid(mesh: &PolyData) -> (usize, PolyData) {
    let n = mesh.points.len();
    if n == 0 {
        return (0, mesh.clone());
    }
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        if !valid_polygon_cell(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut adj);
        }
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(mesh, strip, &mut adj);
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut adj);
        }
    }
    let mut best_vertex = 0;
    let mut best_reachable = 0usize;
    let mut best_max_dist = f64::INFINITY;
    let mut eccentricity = vec![0.0f64; n];
    for src in 0..n {
        let mut dist = vec![f64::INFINITY; n];
        dist[src] = 0.0;
        let mut heap = BinaryHeap::new();
        heap.push(State {
            cost: 0.0,
            node: src,
        });
        while let Some(State { cost, node }) = heap.pop() {
            if cost > dist[node] {
                continue;
            }
            for &(nb, w) in &adj[node] {
                let next = cost + w;
                if next < dist[nb] {
                    dist[nb] = next;
                    heap.push(State {
                        cost: next,
                        node: nb,
                    });
                }
            }
        }
        let mut reachable = 0usize;
        let mut max_d = 0.0f64;
        for &d in &dist {
            if d.is_finite() {
                reachable += 1;
                max_d = max_d.max(d);
            }
        }
        eccentricity[src] = max_d;
        if reachable > best_reachable || (reachable == best_reachable && max_d < best_max_dist) {
            best_reachable = reachable;
            best_max_dist = max_d;
            best_vertex = src;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Eccentricity",
            eccentricity,
            1,
        )));
    result.point_data_mut().set_active_scalars("Eccentricity");
    (best_vertex, result)
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
    fn test_centroid() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (v, r) = geodesic_centroid(&mesh);
        assert!(v < 3);
        assert!(r.point_data().get_array("Eccentricity").is_some());
    }
}
