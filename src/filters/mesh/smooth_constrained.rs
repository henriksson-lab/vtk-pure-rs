//! Constrained Laplacian smoothing that preserves boundaries and features.

use crate::data::PolyData;

/// Smooth mesh while keeping boundary and feature vertices fixed.
pub fn smooth_preserve_boundary(
    mesh: &PolyData,
    iterations: usize,
    lambda: f64,
    feature_angle_deg: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let cos_feature = feature_angle_deg.to_radians().cos();

    // Build adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    for cell in &cells {
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
                *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }

    // Find fixed vertices (boundary + feature edges)
    let mut fixed = vec![false; n];
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 1 {
            fixed[a] = true;
            fixed[b] = true;
        } else if faces.len() == 2 {
            let n0 = face_normal(&cells[faces[0]], mesh);
            let n1 = face_normal(&cells[faces[1]], mesh);
            let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
            if dot < cos_feature {
                fixed[a] = true;
                fixed[b] = true;
            }
        }
    }

    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if fixed[i] || neighbors[i].is_empty() {
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

    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    result
}

fn face_normal(cell: &[i64], mesh: &PolyData) -> [f64; 3] {
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
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_smooth_preserves_boundary() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = smooth_preserve_boundary(&mesh, 5, 0.5, 30.0);
        // Boundary vertices (0,1,2) should stay fixed
        let p0 = r.points.get(0);
        assert!((p0[0] - 0.0).abs() < 1e-10);
    }
}
