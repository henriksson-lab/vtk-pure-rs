//! Generate geodesic distance contours from a seed vertex.
use crate::data::{CellArray, Points, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
const CONTOUR_EPSILON: f64 = 1e-12;

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

pub fn geodesic_contours(mesh: &PolyData, seed: usize, contour_spacing: f64) -> PolyData {
    let n = mesh.points.len();
    if seed >= n || contour_spacing <= 0.0 {
        return PolyData::new();
    }
    // Dijkstra
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
        for &(next_node, weight) in &nb[node] {
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
    // Extract contour lines
    let max_d = dist
        .iter()
        .filter(|&&d| d.is_finite())
        .cloned()
        .fold(0.0f64, f64::max);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut iso = contour_spacing;
    while iso < max_d {
        for cell in mesh.polys.iter() {
            if !valid_polygon_cell(cell, n) {
                continue;
            }
            add_contour_cell(mesh, cell, &dist, iso, &mut pts, &mut lines);
        }
        for strip in mesh.strips.iter() {
            for (i, tri) in strip.windows(3).enumerate() {
                if !valid_triangle(tri, n) {
                    continue;
                }
                let tri = if i % 2 == 0 {
                    [tri[0], tri[1], tri[2]]
                } else {
                    [tri[1], tri[0], tri[2]]
                };
                add_contour_cell(mesh, &tri, &dist, iso, &mut pts, &mut lines);
            }
        }
        iso += contour_spacing;
    }
    let mut contour_mesh = PolyData::new();
    contour_mesh.points = pts;
    contour_mesh.lines = lines;
    contour_mesh
}

fn add_contour_cell(
    mesh: &PolyData,
    cell: &[i64],
    dist: &[f64],
    iso: f64,
    pts: &mut Points<f64>,
    lines: &mut CellArray,
) {
    let n = mesh.points.len();
    let nc = cell.len();
    let mut edge_pts: Vec<[f64; 3]> = Vec::new();
    for i in 0..nc {
        let Some(a) = valid_point_id(cell[i], n) else {
            continue;
        };
        let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
            continue;
        };
        let da = dist[a];
        let db = dist[b];
        if let Some(point) = contour_edge_point(mesh, a, b, da, db, iso) {
            if !edge_pts.iter().any(|&existing| same_point(existing, point)) {
                edge_pts.push(point);
            }
        }
    }
    if edge_pts.len() == 2 {
        let i0 = pts.len();
        pts.push(edge_pts[0]);
        pts.push(edge_pts[1]);
        lines.push_cell(&[i0 as i64, (i0 + 1) as i64]);
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

fn contour_edge_point(
    mesh: &PolyData,
    a: usize,
    b: usize,
    da: f64,
    db: f64,
    iso: f64,
) -> Option<[f64; 3]> {
    if !da.is_finite() || !db.is_finite() {
        return None;
    }
    let fa = da - iso;
    let fb = db - iso;
    if fa.abs() <= CONTOUR_EPSILON {
        return Some(mesh.points.get(a));
    }
    if fb.abs() <= CONTOUR_EPSILON {
        return Some(mesh.points.get(b));
    }
    if fa * fb > 0.0 {
        return None;
    }
    let t = (iso - da) / (db - da);
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    Some([
        pa[0] + t * (pb[0] - pa[0]),
        pa[1] + t * (pb[1] - pa[1]),
        pa[2] + t * (pb[2] - pa[2]),
    ])
}

fn same_point(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= CONTOUR_EPSILON
        && (a[1] - b[1]).abs() <= CONTOUR_EPSILON
        && (a[2] - b[2]).abs() <= CONTOUR_EPSILON
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
        let r = geodesic_contours(&m, 0, 1.0);
        assert!(r.lines.num_cells() >= 1);
    }
}
