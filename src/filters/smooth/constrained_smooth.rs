//! Constrained Laplacian smoothing.
//!
//! Smooths mesh vertices while respecting constraints: boundary vertices
//! are fixed, and vertices can be constrained to move only along their
//! normal direction or within a maximum displacement.

use crate::data::{Points, PolyData};

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
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let original: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let adj = build_adjacency(mesh, n);
    let max_d2 = max_displacement * max_displacement;

    let mut positions = original.clone();

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
            for c in 0..3 {
                new_pos[i][c] = positions[i][c] * (1.0 - factor) + (avg[c] / k) * factor;
            }

            // Clamp displacement from original
            let dx = new_pos[i][0] - original[i][0];
            let dy = new_pos[i][1] - original[i][1];
            let dz = new_pos[i][2] - original[i][2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 > max_d2 {
                let scale = max_displacement / d2.sqrt();
                for c in 0..3 {
                    new_pos[i][c] = original[i][c] + (new_pos[i][c] - original[i][c]) * scale;
                }
            }
        }
        positions = new_pos;
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
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            adj[a].insert(b);
            adj[b].insert(a);
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn find_boundary_vertices(mesh: &PolyData) -> Vec<bool> {
    let n = mesh.points.len();
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
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
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let fn_ = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        for &pid in cell {
            let idx = pid as usize;
            for c in 0..3 {
                normals[idx][c] += fn_[c];
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
