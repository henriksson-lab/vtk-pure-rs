//! Volume-preserving Laplacian smoothing.

use crate::data::PolyData;

/// Volume-preserving smooth: Laplacian smooth followed by uniform scale to restore volume.
pub fn smooth_volume_preserving(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let orig_vol = signed_volume(mesh).abs();
    if orig_vol < 1e-30 || mesh.points.len() == 0 {
        return mesh.clone();
    }

    let n = mesh.points.len();
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

    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let mut avg = [0.0, 0.0, 0.0];
            for &nb in &neighbors[i] {
                avg[0] += positions[nb][0];
                avg[1] += positions[nb][1];
                avg[2] += positions[nb][2];
            }
            let k = neighbors[i].len() as f64;
            new_pos[i][0] = positions[i][0] + lambda * (avg[0] / k - positions[i][0]);
            new_pos[i][1] = positions[i][1] + lambda * (avg[1] / k - positions[i][1]);
            new_pos[i][2] = positions[i][2] + lambda * (avg[2] / k - positions[i][2]);
        }
        positions = new_pos;
    }

    // Compute new volume and scale to restore
    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    let new_vol = signed_volume(&result).abs();
    if new_vol > 1e-30 {
        let scale = (orig_vol / new_vol).powf(1.0 / 3.0);
        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for i in 0..n {
            let p = result.points.get(i);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        cx /= n as f64;
        cy /= n as f64;
        cz /= n as f64;
        // Scale from centroid
        for i in 0..n {
            let p = result.points.get(i);
            result.points.set(
                i,
                [
                    cx + (p[0] - cx) * scale,
                    cy + (p[1] - cy) * scale,
                    cz + (p[2] - cz) * scale,
                ],
            );
        }
    }
    result
}

fn signed_volume(mesh: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
    }
    vol / 6.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_volume_preserved() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let orig_vol = signed_volume(&mesh).abs();
        let smoothed = smooth_volume_preserving(&mesh, 3, 0.3);
        let new_vol = signed_volume(&smoothed).abs();
        assert!(
            (orig_vol - new_vol).abs() / orig_vol.max(1e-10) < 0.1,
            "volume changed: {orig_vol} vs {new_vol}"
        );
    }
}
