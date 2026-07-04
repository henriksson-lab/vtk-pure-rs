//! Volume-preserving mesh smoothing.

use crate::data::{Points, PolyData};

/// Smooth mesh while preserving total volume.
///
/// After each smoothing step, rescales the mesh to maintain original volume.
pub fn smooth_preserve_volume(mesh: &PolyData, factor: f64, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let original_vol = signed_volume(mesh).abs();
    if original_vol < 1e-15 {
        return mesh.clone();
    }

    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        // Laplacian smooth
        let mut new_pos = pos.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += pos[j][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] = pos[i][c] * (1.0 - factor) + (avg[c] / k) * factor;
            }
        }
        pos = new_pos;

        // Compute current volume and rescale
        let mut temp = mesh.clone();
        temp.points = Points::from(pos.clone());
        let new_vol = signed_volume(&temp).abs();
        if new_vol > 1e-15 {
            let scale = (original_vol / new_vol).powf(1.0 / 3.0);
            // Find centroid
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for p in &pos {
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let nf = n as f64;
            cx /= nf;
            cy /= nf;
            cz /= nf;
            for p in pos.iter_mut() {
                p[0] = cx + (p[0] - cx) * scale;
                p[1] = cy + (p[1] - cy) * scale;
                p[2] = cz + (p[2] - cz) * scale;
            }
        }
    }

    let mut result = mesh.clone();
    result.points = Points::from(pos);
    result
}

fn signed_volume(mesh: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a_idx = cell[0] as usize;
        if a_idx >= mesh.points.len() {
            continue;
        }
        let a = mesh.points.get(a_idx);
        for i in 1..cell.len() - 1 {
            let b_idx = cell[i] as usize;
            let c_idx = cell[i + 1] as usize;
            if b_idx >= mesh.points.len() || c_idx >= mesh.points.len() {
                continue;
            }
            let b = mesh.points.get(b_idx);
            let c = mesh.points.get(c_idx);
            vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
    }
    vol / 6.0
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let x = c[i] as usize;
            let y = c[(i + 1) % nc] as usize;
            if x < n && y < n {
                a[x].insert(y);
                a[y].insert(x);
            }
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volume_preserved() {
        // Closed cube
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        );
        let orig_vol = signed_volume(&mesh).abs();
        let result = smooth_preserve_volume(&mesh, 0.3, 5);
        let new_vol = signed_volume(&result).abs();
        assert!(
            (new_vol - orig_vol).abs() / orig_vol < 0.05,
            "vol changed: {orig_vol} -> {new_vol}"
        );
    }
}
