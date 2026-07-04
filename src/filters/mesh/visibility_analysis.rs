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
        for i in 0..n {
            if is_visible(mesh, *vp, i) {
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
    let mut count = 0;
    for i in 0..n {
        if is_visible(mesh, viewpoint, i) {
            count += 1;
        }
    }
    count
}

fn is_visible(mesh: &PolyData, viewpoint: [f64; 3], vertex_id: usize) -> bool {
    let point = mesh.points.get(vertex_id);
    let dir = [
        point[0] - viewpoint[0],
        point[1] - viewpoint[1],
        point[2] - viewpoint[2],
    ];
    let distance = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if distance < 1e-15 {
        return true;
    }
    let dir = [dir[0] / distance, dir[1] / distance, dir[2] / distance];
    let target_tolerance = (distance * 1e-9).max(1e-9);
    let mut closest = f64::MAX;

    for cell in mesh.polys.iter() {
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let tri = [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize];
            let a = mesh.points.get(tri[0]);
            let b = mesh.points.get(tri[1]);
            let c = mesh.points.get(tri[2]);
            if let Some(t) = ray_triangle(viewpoint, dir, a, b, c) {
                if t < closest {
                    closest = t;
                }
            }
        }
    }

    !closest.is_finite() || closest + target_tolerance >= distance
}

fn ray_triangle(
    origin: [f64; 3],
    dir: [f64; 3],
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
) -> Option<f64> {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    if t > 1e-12 {
        Some(t)
    } else {
        None
    }
}

fn valid_cell(cell: &[i64], num_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
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
    fn isolated_point_visible_without_polygon_hit() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);

        assert_eq!(visibility_fraction(&mesh, [0.0, 0.0, 1.0]), 1.0);
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
