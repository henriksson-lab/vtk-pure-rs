//! Bilateral normal filtering: smooth face normals then update vertex positions.

use crate::data::{Points, PolyData};

/// Bilateral normal filtering for mesh denoising.
///
/// First smooths face normals bilaterally, then updates vertex positions
/// to match the smoothed normals. Preserves sharp features.
pub fn bilateral_normal_filter(
    mesh: &PolyData,
    normal_iterations: usize,
    vertex_iterations: usize,
    sigma_s: f64,
    sigma_r: f64,
) -> PolyData {
    let n_points = mesh.points.len();
    let all_cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= 3 && c.iter().all(|&pid| valid_point_id(pid, n_points).is_some()))
        .map(|c| c.to_vec())
        .collect();
    let nc = all_cells.len();
    if nc == 0 || sigma_s <= 0.0 || sigma_r <= 0.0 {
        return mesh.clone();
    }

    // Compute face normals and centroids
    let mut normals: Vec<[f64; 3]> = all_cells.iter().map(|c| face_normal(mesh, c)).collect();
    let centroids: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|c| {
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &pid in c {
                let p = mesh.points.get(pid as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let k = c.len() as f64;
            [cx / k, cy / k, cz / k]
        })
        .collect();

    // Build face adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // Step 1: Bilateral filter on face normals
    for _ in 0..normal_iterations {
        let mut new_normals = normals.clone();
        for ci in 0..nc {
            let mut wn = [0.0; 3];
            let mut w_sum = 0.0;
            let cell = &all_cells[ci];
            let n = cell.len();
            for i in 0..n {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % n] as usize;
                if let Some(nbs) = edge_faces.get(&(a.min(b), a.max(b))) {
                    for &ni in nbs {
                        if ni == ci {
                            continue;
                        }
                        let ds = dist(&centroids[ci], &centroids[ni]);
                        let dn = 1.0 - dot(normals[ci], normals[ni]).clamp(-1.0, 1.0);
                        let w = (-ds * ds / (2.0 * sigma_s * sigma_s)
                            - dn * dn / (2.0 * sigma_r * sigma_r))
                            .exp();
                        for j in 0..3 {
                            wn[j] += w * normals[ni][j];
                        }
                        w_sum += w;
                    }
                }
            }
            if w_sum > 1e-15 {
                for j in 0..3 {
                    wn[j] /= w_sum;
                }
                let len = (wn[0] * wn[0] + wn[1] * wn[1] + wn[2] * wn[2]).sqrt();
                if len > 1e-15 {
                    new_normals[ci] = [wn[0] / len, wn[1] / len, wn[2] / len];
                }
            }
        }
        normals = new_normals;
    }

    // Step 2: Update vertex positions to match smoothed normals
    let mut pos: Vec<[f64; 3]> = (0..n_points).map(|i| mesh.points.get(i)).collect();
    for _ in 0..vertex_iterations {
        let mut new_pos = pos.clone();
        let mut displacements = vec![[0.0; 3]; n_points];
        let mut counts = vec![0usize; n_points];
        for (ci, cell) in all_cells.iter().enumerate() {
            let centroid = {
                let mut c = [0.0; 3];
                for &pid in cell {
                    for j in 0..3 {
                        c[j] += pos[pid as usize][j];
                    }
                }
                let k = cell.len() as f64;
                [c[0] / k, c[1] / k, c[2] / k]
            };
            for &pid in cell {
                let vi = pid as usize;
                let d = [
                    pos[vi][0] - centroid[0],
                    pos[vi][1] - centroid[1],
                    pos[vi][2] - centroid[2],
                ];
                let proj = d[0] * normals[ci][0] + d[1] * normals[ci][1] + d[2] * normals[ci][2];
                for j in 0..3 {
                    displacements[vi][j] -= 0.5 * proj * normals[ci][j];
                }
                counts[vi] += 1;
            }
        }
        for vi in 0..n_points {
            if counts[vi] == 0 {
                continue;
            }
            let scale = 1.0 / counts[vi] as f64;
            for j in 0..3 {
                new_pos[vi][j] += displacements[vi][j] * scale;
            }
        }
        pos = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(pos);
    result
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}
fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&idx| idx < n_points)
}
fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn denoise() {
        let mut pts = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                let z = if x == 4 && y == 4 { 0.5 } else { 0.0 } + (x as f64 * 0.01).sin() * 0.05; // noise
                pts.push([x as f64, y as f64, z]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..7 {
            for x in 0..7 {
                let bl = y * 8 + x;
                tris.push([bl, bl + 1, bl + 9]);
                tris.push([bl, bl + 9, bl + 8]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = bilateral_normal_filter(&mesh, 3, 3, 2.0, 0.5);
        assert_eq!(result.points.len(), mesh.points.len());
    }
}
