//! Convexity analysis: measure how convex a mesh is and find concave regions.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-edge convexity: +1 = convex, -1 = concave, 0 = flat.
pub fn edge_convexity(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells.iter().map(|c| face_normal(mesh, c)).collect();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // Per-vertex convexity score
    let mut convexity = vec![0.0f64; n];
    let mut counts = vec![0usize; n];

    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let n0 = normals[faces[0]];
        let n1 = normals[faces[1]];
        // Check if edge is convex or concave
        let c0 = cell_centroid(mesh, &all_cells[faces[0]]);
        let c1 = cell_centroid(mesh, &all_cells[faces[1]]);
        let sign = edge_convexity_sign(n0, n1, c0, c1);

        convexity[a] += sign;
        counts[a] += 1;
        convexity[b] += sign;
        counts[b] += 1;
    }

    for i in 0..n {
        if counts[i] > 0 {
            convexity[i] /= counts[i] as f64;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Convexity",
            convexity,
            1,
        )));
    result
}

/// Compute overall convexity ratio: fraction of convex edges.
pub fn convexity_ratio(mesh: &PolyData) -> f64 {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells.iter().map(|c| face_normal(mesh, c)).collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut convex_count = 0;
    let mut total = 0;
    for (_, faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        total += 1;
        let c0 = cell_centroid(mesh, &all_cells[faces[0]]);
        let c1 = cell_centroid(mesh, &all_cells[faces[1]]);
        if edge_convexity_sign(normals[faces[0]], normals[faces[1]], c0, c1) > 0.0 {
            convex_count += 1;
        }
    }
    if total > 0 {
        convex_count as f64 / total as f64
    } else {
        1.0
    }
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

fn edge_convexity_sign(n0: [f64; 3], n1: [f64; 3], c0: [f64; 3], c1: [f64; 3]) -> f64 {
    let c0_to_c1 = [c1[0] - c0[0], c1[1] - c0[1], c1[2] - c0[2]];
    let c1_to_c0 = [-c0_to_c1[0], -c0_to_c1[1], -c0_to_c1[2]];
    let d0 = n0[0] * c0_to_c1[0] + n0[1] * c0_to_c1[1] + n0[2] * c0_to_c1[2];
    let d1 = n1[0] * c1_to_c0[0] + n1[1] * c1_to_c0[1] + n1[2] * c1_to_c0[2];
    if d0.abs() < 1e-12 && d1.abs() < 1e-12 {
        0.0
    } else if d0 <= 1e-12 && d1 <= 1e-12 {
        1.0
    } else {
        -1.0
    }
}

fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut c = [0.0; 3];
    for &pid in cell {
        let p = mesh.points.get(pid as usize);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let k = cell.len() as f64;
    [c[0] / k, c[1] / k, c[2] / k]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convex_sphere() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let ratio = convexity_ratio(&mesh);
        assert!(ratio > 0.8, "sphere should be mostly convex, got {ratio}");
    }
    #[test]
    fn per_vertex() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = edge_convexity(&mesh);
        assert!(result.point_data().get_array("Convexity").is_some());
    }
}
