//! Constrained Laplacian smoothing that preserves boundaries and features.

use crate::data::PolyData;
use std::collections::{HashMap, HashSet};

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

    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let stencils = build_stencils(&cells, n);
    let fixed = fixed_vertices(mesh, &cells, feature_angle_deg);
    let mut constraint2 = vec![f64::MAX; n];
    for i in 0..n {
        if fixed[i] {
            constraint2[i] = 0.0;
        }
    }
    smooth_with_constraints(mesh, &stencils, &constraint2, iterations, lambda, 0.0)
}

fn build_stencils(cells: &[Vec<i64>], n: usize) -> Vec<Vec<usize>> {
    let mut stencils: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in cells {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            if let (Some(a), Some(b)) = (
                valid_point_id(cell[i], n),
                valid_point_id(cell[(i + 1) % nc], n),
            ) {
                stencils[a].insert(b);
                stencils[b].insert(a);
            }
        }
    }
    stencils
        .into_iter()
        .map(|s| s.into_iter().collect())
        .collect()
}

fn fixed_vertices(mesh: &PolyData, cells: &[Vec<i64>], feature_angle_deg: f64) -> Vec<bool> {
    let n = mesh.points.len();
    let cos_feature = feature_angle_deg.to_radians().cos();
    let mut fixed = vec![false; n];
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            if let (Some(a), Some(b)) = (
                valid_point_id(cell[i], n),
                valid_point_id(cell[(i + 1) % nc], n),
            ) {
                edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
            }
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
    fixed
}

fn smooth_with_constraints(
    mesh: &PolyData,
    stencils: &[Vec<usize>],
    constraint2: &[f64],
    iterations: usize,
    relaxation_factor: f64,
    convergence: f64,
) -> PolyData {
    let n = mesh.points.len();
    let original: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut positions = original.clone();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        let mut max_distance2 = 0.0;

        for i in 0..n {
            if stencils[i].is_empty() || constraint2[i] == 0.0 {
                new_pos[i] = original[i];
                continue;
            }

            let mut avg = [0.0; 3];
            for &nb in &stencils[i] {
                avg[0] += positions[nb][0];
                avg[1] += positions[nb][1];
                avg[2] += positions[nb][2];
            }
            let k = stencils[i].len() as f64;
            let mut x = [
                positions[i][0] + relaxation_factor * (avg[0] / k - positions[i][0]),
                positions[i][1] + relaxation_factor * (avg[1] / k - positions[i][1]),
                positions[i][2] + relaxation_factor * (avg[2] / k - positions[i][2]),
            ];

            let d2 = distance2(x, original[i]);
            if d2 > constraint2[i] {
                let t = (constraint2[i] / d2).sqrt();
                x = [
                    original[i][0] + t * (x[0] - original[i][0]),
                    original[i][1] + t * (x[1] - original[i][1]),
                    original[i][2] + t * (x[2] - original[i][2]),
                ];
            }

            max_distance2 = f64::max(max_distance2, distance2(x, positions[i]));
            new_pos[i] = x;
        }
        positions = new_pos;
        if max_distance2.sqrt() <= convergence {
            break;
        }
    }

    let mut result = mesh.clone();
    for i in 0..n {
        result.points.set(i, positions[i]);
    }
    result
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn face_normal(cell: &[i64], mesh: &PolyData) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let npts = mesh.points.len();
    let Some(a_id) = valid_point_id(cell[0], npts) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(b_id) = valid_point_id(cell[1], npts) else {
        return [0.0, 0.0, 1.0];
    };
    let Some(c_id) = valid_point_id(cell[2], npts) else {
        return [0.0, 0.0, 1.0];
    };
    let a = mesh.points.get(a_id);
    let b = mesh.points.get(b_id);
    let c = mesh.points.get(c_id);
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

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
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

    #[test]
    fn skips_malformed_cells() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.polys.push_cell(&[0, -1, 99]);

        let result = smooth_preserve_boundary(&mesh, 1, 0.5, 30.0);
        assert_eq!(result.points.len(), mesh.points.len());
    }
}
