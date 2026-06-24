//! Bilateral mesh smoothing (edge-preserving).

use crate::data::PolyData;

/// Bilateral mesh smoothing: smooths positions while preserving sharp features.
pub fn smooth_bilateral_mesh(
    mesh: &PolyData,
    iterations: usize,
    sigma_spatial: f64,
    sigma_normal: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    // Build adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }

    // Compute vertex normals
    let normals = compute_normals(mesh);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let ss2 = 2.0 * sigma_spatial * sigma_spatial;
    let sn2 = 2.0 * sigma_normal * sigma_normal;

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let ni = normals[i];
            let mut sum = [0.0, 0.0, 0.0];
            let mut wsum = 0.0;
            for &nb in &neighbors[i] {
                let dp = [
                    positions[nb][0] - positions[i][0],
                    positions[nb][1] - positions[i][1],
                    positions[nb][2] - positions[i][2],
                ];
                let dist2 = dp[0] * dp[0] + dp[1] * dp[1] + dp[2] * dp[2];
                let h = dp[0] * ni[0] + dp[1] * ni[1] + dp[2] * ni[2]; // height along normal
                let ws = (-dist2 / ss2).exp();
                let wn = (-h * h / sn2).exp();
                let w = ws * wn;
                sum[0] += w * dp[0];
                sum[1] += w * dp[1];
                sum[2] += w * dp[2];
                wsum += w;
            }
            if wsum > 1e-15 {
                new_pos[i][0] = positions[i][0] + sum[0] / wsum;
                new_pos[i][1] = positions[i][1] + sum[1] / wsum;
                new_pos[i][2] = positions[i][2] + sum[2] / wsum;
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    result
}

fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut normals = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                normals[vi][0] += fn_[0];
                normals[vi][1] += fn_[1];
                normals[vi][2] += fn_[2];
            }
        }
    }
    for nm in &mut normals {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-15 {
            nm[0] /= len;
            nm[1] /= len;
            nm[2] /= len;
        }
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bilateral() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = smooth_bilateral_mesh(&mesh, 3, 1.0, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
