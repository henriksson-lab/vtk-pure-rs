//! Extract silhouette edges from a viewpoint.
use crate::data::{CellArray, PolyData};
pub fn extract_silhouette(mesh: &PolyData, view_point: [f64; 3]) -> PolyData {
    let mut ef: std::collections::HashMap<(usize, usize), TwoNormals> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        insert_polygon_edges(&mut ef, mesh, cell, mesh.points.len());
    }
    let mut lines = CellArray::new();
    let feature_angle_cos = 60.0_f64.to_radians().cos();
    for (&(a, b), normals) in &ef {
        let winged = norm(normals.left) > 0.5 && norm(normals.right) > 0.5;
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let view = [
            view_point[0] - 0.5 * (pa[0] + pb[0]),
            view_point[1] - 0.5 * (pa[1] + pb[1]),
            view_point[2] - 0.5 * (pa[2] + pb[2]),
        ];
        let d1 = dot(view, normals.left);
        let d2 = dot(view, normals.right);
        let edge_angle_cos = dot(normals.left, normals.right);
        let is_silhouette = (winged && d1 * d2 < 0.0) || edge_angle_cos < feature_angle_cos;
        if is_silhouette {
            lines.push_cell(&[a as i64, b as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = mesh.points.clone();
    r.lines = lines;
    *r.field_data_mut() = mesh.field_data().clone();
    r
}

#[derive(Clone, Copy, Default)]
struct TwoNormals {
    left: [f64; 3],
    right: [f64; 3],
}

fn insert_polygon_edges(
    ef: &mut std::collections::HashMap<(usize, usize), TwoNormals>,
    mesh: &PolyData,
    cell: &[i64],
    n_points: usize,
) {
    if cell.len() < 3 {
        return;
    }
    let normal = polygon_normal(mesh, cell);
    for i in 0..cell.len() {
        insert_edge(ef, cell[i], cell[(i + 1) % cell.len()], normal, n_points);
    }
}

fn insert_edge(
    ef: &mut std::collections::HashMap<(usize, usize), TwoNormals>,
    a: i64,
    b: i64,
    normal: [f64; 3],
    n_points: usize,
) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a != b {
        let entry = ef.entry((a.min(b), a.max(b))).or_default();
        if a < b {
            entry.left = normal;
        } else {
            entry.right = normal;
        }
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut normal = [0.0; 3];
    for i in 0..cell.len() {
        let Some(a) = valid_point_index(cell[i], mesh.points.len()) else {
            return [0.0; 3];
        };
        let Some(b) = valid_point_index(cell[(i + 1) % cell.len()], mesh.points.len()) else {
            return [0.0; 3];
        };
        let p = mesh.points.get(a);
        let q = mesh.points.get(b);
        normal[0] += (p[1] - q[1]) * (p[2] + q[2]);
        normal[1] += (p[2] - q[2]) * (p[0] + q[0]);
        normal[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let length = norm(normal);
    if length > 0.0 {
        [normal[0] / length, normal[1] / length, normal[2] / length]
    } else {
        [0.0; 3]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = extract_silhouette(&m, [0.0, 0.0, 10.0]);
        assert!(r.lines.num_cells() >= 1);
    }
}
