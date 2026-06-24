//! Mesh visibility analysis from viewpoints.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute what fraction of vertices are visible from a viewpoint.
pub fn visibility_fraction(mesh: &PolyData, viewpoint: [f64; 3]) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
    }
    let visible = count_visible(mesh, viewpoint);
    visible as f64 / n as f64
}

/// Compute visibility from multiple viewpoints and sum.
///
/// Adds a "VisibilityCount" array: how many viewpoints can see each vertex.
pub fn multi_view_visibility(mesh: &PolyData, viewpoints: &[[f64; 3]]) -> PolyData {
    let n = mesh.points.len();
    let mut counts = vec![0.0f64; n];

    for vp in viewpoints {
        let normals = compute_face_normals(mesh);
        // Simplified: vertex is visible if its normal faces the viewpoint
        for i in 0..n {
            let p = mesh.points.get(i);
            let view_dir = [vp[0] - p[0], vp[1] - p[1], vp[2] - p[2]];
            let nm = vertex_normal(mesh, i, &normals);
            let dot = view_dir[0] * nm[0] + view_dir[1] * nm[1] + view_dir[2] * nm[2];
            if dot > 0.0 {
                counts[i] += 1.0;
            }
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VisibilityCount",
            counts,
            1,
        )));
    result
}

/// Find the optimal viewpoint (from a set of candidates) that maximizes visibility.
pub fn best_viewpoint(mesh: &PolyData, candidates: &[[f64; 3]]) -> ([f64; 3], f64) {
    let mut best_vp = [0.0; 3];
    let mut best_frac = 0.0;
    for &vp in candidates {
        let frac = visibility_fraction(mesh, vp);
        if frac > best_frac {
            best_frac = frac;
            best_vp = vp;
        }
    }
    (best_vp, best_frac)
}

fn count_visible(mesh: &PolyData, viewpoint: [f64; 3]) -> usize {
    let n = mesh.points.len();
    let normals = compute_face_normals(mesh);
    let mut count = 0;
    for i in 0..n {
        let p = mesh.points.get(i);
        let view_dir = [
            viewpoint[0] - p[0],
            viewpoint[1] - p[1],
            viewpoint[2] - p[2],
        ];
        let nm = vertex_normal(mesh, i, &normals);
        if view_dir[0] * nm[0] + view_dir[1] * nm[1] + view_dir[2] * nm[2] > 0.0 {
            count += 1;
        }
    }
    count
}

fn compute_face_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    mesh.polys
        .iter()
        .map(|cell| {
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
        })
        .collect()
}

fn vertex_normal(mesh: &PolyData, vi: usize, face_normals: &[[f64; 3]]) -> [f64; 3] {
    let mut nm = [0.0; 3];
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.iter().any(|&pid| pid as usize == vi) {
            if ci < face_normals.len() {
                for c in 0..3 {
                    nm[c] += face_normals[ci][c];
                }
            }
        }
    }
    let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
    if len > 1e-15 {
        [nm[0] / len, nm[1] / len, nm[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_visible() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let frac = visibility_fraction(&mesh, [0.5, 0.5, 1.0]); // in front
        assert!(frac > 0.5);
    }
    #[test]
    fn multi_view() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = multi_view_visibility(&mesh, &[[0.5, 0.5, 1.0], [0.5, 0.5, -1.0]]);
        assert!(result.point_data().get_array("VisibilityCount").is_some());
    }
    #[test]
    fn best_vp() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (vp, frac) = best_viewpoint(
            &mesh,
            &[[0.5, 0.5, 1.0], [0.5, 0.5, -1.0], [10.0, 0.0, 0.0]],
        );
        assert!(frac > 0.0);
    }
}
