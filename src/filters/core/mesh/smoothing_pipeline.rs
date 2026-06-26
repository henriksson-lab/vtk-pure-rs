//! Mesh smoothing pipeline combining multiple smoothing passes.

use crate::data::{CellArray, Points, PolyData};

/// Adaptive smoothing: stronger smoothing where curvature is high.
pub fn adaptive_smooth(mesh: &PolyData, iterations: usize, max_factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        // Compute per-vertex curvature estimate
        let curvatures: Vec<f64> = (0..n)
            .map(|i| {
                if adj[i].is_empty() {
                    return 0.0;
                }
                let mut lap = [0.0; 3];
                for &j in &adj[i] {
                    for c in 0..3 {
                        lap[c] += positions[j][c] - positions[i][c];
                    }
                }
                let k = adj[i].len() as f64;
                ((lap[0] / k).powi(2) + (lap[1] / k).powi(2) + (lap[2] / k).powi(2)).sqrt()
            })
            .collect();

        let max_curv = curvatures.iter().cloned().fold(0.0f64, f64::max).max(1e-15);

        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let factor = (curvatures[i] / max_curv) * max_factor;
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[j][c];
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

/// Progressive smoothing: apply increasing smoothing factors.
pub fn progressive_smooth(mesh: &PolyData, n_steps: usize, final_factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for step in 0..n_steps {
        let factor = final_factor * (step + 1) as f64 / n_steps as f64;
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += positions[j][c];
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

/// Anisotropic smoothing: smooth more along low-curvature directions.
pub fn anisotropic_smooth(mesh: &PolyData, iterations: usize, factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }

    let adj = build_adj(mesh, n);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    // Compute vertex normals once
    let normals = compute_normals(mesh);

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let ni = &normals[i];
            let mut tangent_disp = [0.0; 3];
            let mut count = 0.0;

            for &j in &adj[i] {
                let d = [
                    positions[j][0] - positions[i][0],
                    positions[j][1] - positions[i][1],
                    positions[j][2] - positions[i][2],
                ];
                // Project displacement onto tangent plane
                let dot_n = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
                tangent_disp[0] += d[0] - dot_n * ni[0];
                tangent_disp[1] += d[1] - dot_n * ni[1];
                tangent_disp[2] += d[2] - dot_n * ni[2];
                count += 1.0;
            }

            if count > 0.0 {
                for c in 0..3 {
                    new_pos[i][c] += factor * tangent_disp[c] / count;
                }
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    add_adj_cells(&mesh.lines, n, false, &mut adj);
    add_adj_cells(&mesh.polys, n, true, &mut adj);
    add_adj_cells(&mesh.strips, n, false, &mut adj);
    adj.into_iter()
        .map(|s| {
            let mut neighbors: Vec<usize> = s.into_iter().collect();
            neighbors.sort_unstable();
            neighbors
        })
        .collect()
}

fn add_adj_cells(
    cells: &CellArray,
    n_points: usize,
    closed: bool,
    adj: &mut [std::collections::HashSet<usize>],
) {
    for cell in cells.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        let edge_count = if closed { nc } else { nc - 1 };
        for i in 0..edge_count {
            let Some(a) = valid_point_id(cell[i], n_points) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n_points) else {
                continue;
            };
            if a != b {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&idx| idx < n_points)
}

fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut normals = vec![[0.0; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(ia) = valid_point_id(cell[0], n) else {
            continue;
        };
        let Some(ib) = valid_point_id(cell[1], n) else {
            continue;
        };
        let Some(ic) = valid_point_id(cell[2], n) else {
            continue;
        };
        let a = mesh.points.get(ia);
        let b = mesh.points.get(ib);
        let c = mesh.points.get(ic);
        let fn_ = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        for &pid in cell {
            let Some(idx) = valid_point_id(pid, n) else {
                continue;
            };
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

    fn make_bumpy_plane() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                let z = if x == 5 && y == 5 { 1.0 } else { 0.0 };
                pts.push([x as f64, y as f64, z]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn adaptive() {
        let mesh = make_bumpy_plane();
        let result = adaptive_smooth(&mesh, 3, 0.5);
        // Bump should be reduced
        let z_center = result.points.get(55)[2]; // center point
        assert!(z_center < 1.0);
    }

    #[test]
    fn progressive() {
        let mesh = make_bumpy_plane();
        let result = progressive_smooth(&mesh, 5, 0.5);
        assert_eq!(result.points.len(), mesh.points.len());
    }

    #[test]
    fn anisotropic() {
        let mesh = make_bumpy_plane();
        let result = anisotropic_smooth(&mesh, 3, 0.5);
        assert_eq!(result.points.len(), mesh.points.len());
    }

    #[test]
    fn anisotropic_skips_invalid_cells() {
        let mut mesh = make_bumpy_plane();
        mesh.polys.push_cell(&[0, 1, 999]);
        mesh.polys.push_cell(&[0, -1, 2]);

        let result = anisotropic_smooth(&mesh, 1, 0.5);
        assert_eq!(result.points.len(), mesh.points.len());
    }

    #[test]
    fn progressive_zero_steps_is_identity() {
        let mesh = make_bumpy_plane();
        let result = progressive_smooth(&mesh, 0, 0.5);
        assert_eq!(result.points.len(), mesh.points.len());
        for i in 0..mesh.points.len() {
            assert_eq!(result.points.get(i), mesh.points.get(i));
        }
    }
}
