//! Constrained Laplacian smoothing.
//!
//! Smooths mesh vertices while respecting constraints: boundary vertices
//! are fixed, and vertices can be constrained to move only along their
//! normal direction or within a maximum displacement.

use crate::data::{Points, PolyData};
use std::collections::{HashMap, HashSet};

/// Smooth with boundary vertices fixed (cannot move).
pub fn smooth_constrained_boundary(mesh: &PolyData, iterations: usize, factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let boundary = find_boundary_vertices(mesh);
    let adj = build_adjacency(mesh, n);

    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if boundary[i] || adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &ni in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[ni][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] = positions[i][c] * (1.0 - factor) + (avg[c] / k) * factor;
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

/// Smooth with maximum displacement constraint.
///
/// No vertex moves more than `max_displacement` from its original position.
pub fn smooth_constrained_displacement(
    mesh: &PolyData,
    iterations: usize,
    factor: f64,
    max_displacement: f64,
) -> PolyData {
    smooth_constrained_distance(mesh, iterations, factor, max_displacement, 0.0)
}

/// VTK-style constrained smoothing with a spherical constraint distance.
///
/// This maps the core of `vtkConstrainedSmoothingFilter`: each point moves
/// toward the average of its edge-connected stencil by `relaxation_factor`,
/// the displacement is clamped to a sphere around the original point, and
/// iterations stop when `convergence` is reached or `iterations` is exhausted.
pub fn smooth_constrained_distance(
    mesh: &PolyData,
    iterations: usize,
    relaxation_factor: f64,
    constraint_distance: f64,
    convergence: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || iterations == 0 {
        return mesh.clone();
    }

    let original: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let adj = build_adjacency(mesh, n);
    let constraint2 = constraint_distance.max(0.0) * constraint_distance.max(0.0);

    let mut positions = original.clone();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        let mut max_distance2 = 0.0;
        for i in 0..n {
            if adj[i].is_empty() || constraint2 == 0.0 {
                new_pos[i] = original[i];
                continue;
            }
            let mut avg = [0.0; 3];
            for &ni in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[ni][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] =
                    positions[i][c] + relaxation_factor * (avg[c] / k - positions[i][c]);
            }

            let dx = new_pos[i][0] - original[i][0];
            let dy = new_pos[i][1] - original[i][1];
            let dz = new_pos[i][2] - original[i][2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 > constraint2 {
                let scale = (constraint2 / d2).sqrt();
                for c in 0..3 {
                    new_pos[i][c] = original[i][c] + (new_pos[i][c] - original[i][c]) * scale;
                }
            }
            max_distance2 = f64::max(max_distance2, distance2(new_pos[i], positions[i]));
        }
        positions = new_pos;
        if max_distance2.sqrt() <= convergence {
            break;
        }
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

/// Smooth along normal direction only (tangential smoothing preserved).
pub fn smooth_constrained_normal(mesh: &PolyData, iterations: usize, factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let adj = build_adjacency(mesh, n);
    let normals = compute_vertex_normals(mesh);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &ni in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[ni][c];
                }
            }
            let k = adj[i].len() as f64;
            let target = [avg[0] / k, avg[1] / k, avg[2] / k];

            // Project displacement onto normal
            let disp = [
                target[0] - positions[i][0],
                target[1] - positions[i][1],
                target[2] - positions[i][2],
            ];
            let n_dir = &normals[i];
            let proj = disp[0] * n_dir[0] + disp[1] * n_dir[1] + disp[2] * n_dir[2];
            for c in 0..3 {
                new_pos[i][c] = positions[i][c] + factor * proj * n_dir[c];
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

fn build_adjacency(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in mesh.polys.iter() {
        add_closed_cell_edges(cell, n, &mut adj);
    }
    for cell in mesh.lines.iter() {
        add_open_cell_edges(cell, n, &mut adj);
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn find_boundary_vertices(mesh: &PolyData) -> Vec<bool> {
    let n = mesh.points.len();
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            if let (Some(a), Some(b)) = (
                valid_point_id(cell[i], n),
                valid_point_id(cell[(i + 1) % cell.len()], n),
            ) {
                *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    let mut boundary = vec![false; n];
    for ((a, b), count) in &edge_count {
        if *count == 1 {
            boundary[*a] = true;
            boundary[*b] = true;
        }
    }
    boundary
}

fn compute_vertex_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut normals = vec![[0.0; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let (Some(a_id), Some(b_id), Some(c_id)) = (
            valid_point_id(cell[0], n),
            valid_point_id(cell[1], n),
            valid_point_id(cell[2], n),
        ) else {
            continue;
        };
        let a = mesh.points.get(a_id);
        let b = mesh.points.get(b_id);
        let c = mesh.points.get(c_id);
        let fn_ = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        for &pid in cell {
            if let Some(idx) = valid_point_id(pid, n) {
                for c in 0..3 {
                    normals[idx][c] += fn_[c];
                }
            }
        }
    }
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            for c in 0..3 {
                n[c] /= len;
            }
        }
    }
    normals
}

fn add_closed_cell_edges(cell: &[i64], n: usize, adj: &mut [HashSet<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_edge(cell[i], cell[(i + 1) % cell.len()], n, adj);
    }
}

fn add_open_cell_edges(cell: &[i64], n: usize, adj: &mut [HashSet<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], n, adj);
    }
}

fn add_edge(a: i64, b: i64, n: usize, adj: &mut [HashSet<usize>]) {
    let (Some(a), Some(b)) = (valid_point_id(a, n), valid_point_id(b, n)) else {
        return;
    };
    if a != b {
        adj[a].insert(b);
        adj[b].insert(a);
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_plane() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                let z = if x == 2 && y == 2 { 1.0 } else { 0.0 }; // bump
                pts.push([x as f64, y as f64, z]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn boundary_constrained() {
        let mesh = make_plane();
        let result = smooth_constrained_boundary(&mesh, 5, 0.5);
        // Boundary points should not have moved
        let p0 = result.points.get(0);
        assert!((p0[2] - 0.0).abs() < 1e-10);
        // Center bump should be smoothed
        let center = result.points.get(12); // 2,2
        assert!(center[2] < 1.0);
    }

    #[test]
    fn displacement_constrained() {
        let mesh = make_plane();
        let result = smooth_constrained_displacement(&mesh, 10, 0.5, 0.2);
        // No point should move more than 0.2 from original
        for i in 0..mesh.points.len() {
            let orig = mesh.points.get(i);
            let new_ = result.points.get(i);
            let d = ((new_[0] - orig[0]).powi(2)
                + (new_[1] - orig[1]).powi(2)
                + (new_[2] - orig[2]).powi(2))
            .sqrt();
            assert!(d <= 0.201, "point {i} moved {d}");
        }
    }

    #[test]
    fn normal_constrained() {
        let mesh = make_plane();
        let result = smooth_constrained_normal(&mesh, 3, 0.5);
        assert_eq!(result.points.len(), 25);
    }

    #[test]
    fn empty_mesh() {
        let result = smooth_constrained_boundary(&PolyData::new(), 5, 0.5);
        assert_eq!(result.points.len(), 0);
    }
}
