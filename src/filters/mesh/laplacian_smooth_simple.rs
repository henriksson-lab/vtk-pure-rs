//! Simple Laplacian smoothing with configurable lambda.

use crate::data::PolyData;

/// Laplacian smooth with lambda factor and iteration count.
/// lambda=1.0 is full averaging, lambda=0.5 is half-step.
pub fn laplacian_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
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
            avg[0] /= k;
            avg[1] /= k;
            avg[2] /= k;
            new_pos[i][0] = positions[i][0] + lambda * (avg[0] - positions[i][0]);
            new_pos[i][1] = positions[i][1] + lambda * (avg[1] - positions[i][1]);
            new_pos[i][2] = positions[i][2] + lambda * (avg[2] - positions[i][2]);
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    result
}

/// Taubin smoothing (alternating lambda/mu steps to reduce shrinkage).
pub fn taubin_smooth(mesh: &PolyData, iterations: usize, lambda: f64, mu: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

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

    for iter in 0..iterations * 2 {
        let factor = if iter % 2 == 0 { lambda } else { mu };
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
            avg[0] /= k;
            avg[1] /= k;
            avg[2] /= k;
            new_pos[i][0] = positions[i][0] + factor * (avg[0] - positions[i][0]);
            new_pos[i][1] = positions[i][1] + factor * (avg[1] - positions[i][1]);
            new_pos[i][2] = positions[i][2] + factor * (avg[2] - positions[i][2]);
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_laplacian() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 1.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let smoothed = laplacian_smooth(&mesh, 5, 0.5);
        assert_eq!(smoothed.points.len(), 4);
    }
    #[test]
    fn test_taubin() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 1.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let smoothed = taubin_smooth(&mesh, 3, 0.5, -0.53);
        assert_eq!(smoothed.points.len(), 4);
    }
}
