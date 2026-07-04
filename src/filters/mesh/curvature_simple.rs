//! Simple discrete curvature estimation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian curvature using angle defect.
pub fn gaussian_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut curvature = vec![2.0 * std::f64::consts::PI; n];
    let mut area = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(ids) = valid_cell_prefix(cell, n) else {
            continue;
        };
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let tri_area = triangle_area(p[0], p[1], p[2]);
        let angles = triangle_angles(p);
        for i in 0..3 {
            curvature[ids[i]] -= angles[i];
            area[ids[i]] += tri_area;
        }
    }

    for i in 0..n {
        if area[i] > 0.0 {
            curvature[i] = 3.0 * curvature[i] / area[i];
        } else {
            curvature[i] = 0.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Gauss_Curvature",
            curvature.clone(),
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            curvature,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("Gauss_Curvature");
    result
}

/// Compute discrete mean curvature using VTK-style signed dihedral edges.
pub fn mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut curvature = vec![0.0f64; n];
    let mut num_neighbors = vec![0usize; n];
    let mut edges =
        std::collections::HashMap::<(usize, usize), (usize, usize, [f64; 3], f64)>::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(first_three) = valid_cell_prefix(cell, n) else {
            continue;
        };
        if !cell.iter().all(|&id| valid_point_id(id, n).is_some()) {
            continue;
        }
        let ids: Vec<usize> = cell
            .iter()
            .map(|&id| valid_point_id(id, n).unwrap())
            .collect();
        let first_points = [
            mesh.points.get(first_three[0]),
            mesh.points.get(first_three[1]),
            mesh.points.get(first_three[2]),
        ];
        let first_normal = triangle_normal(first_points[0], first_points[1], first_points[2]);
        let first_area = triangle_area(first_points[0], first_points[1], first_points[2]);

        for i in 0..ids.len() {
            let vl = ids[i];
            let vr = ids[(i + 1) % ids.len()];
            let vo = ids[(i + 2) % ids.len()];
            let key = if vl < vr { (vl, vr) } else { (vr, vl) };
            if let Some((stored_vl, stored_vr, stored_normal, stored_area)) = edges.remove(&key) {
                let start = mesh.points.get(stored_vl);
                let end = mesh.points.get(stored_vr);
                let mut edge = sub(end, start);
                let length = normalize(&mut edge);
                let cs = dot(stored_normal, first_normal);
                let sn = dot(cross(stored_normal, first_normal), edge);
                let mut hf = if sn != 0.0 || cs != 0.0 {
                    length * sn.atan2(cs)
                } else {
                    0.0
                };
                let total_area = stored_area + first_area;
                if total_area != 0.0 {
                    hf = 3.0 * hf / total_area;
                }
                curvature[vl] += hf;
                curvature[vr] += hf;
                num_neighbors[vl] += 1;
                num_neighbors[vr] += 1;
            } else {
                let edge_points = [
                    mesh.points.get(vl),
                    mesh.points.get(vr),
                    mesh.points.get(vo),
                ];
                let edge_normal = triangle_normal(edge_points[0], edge_points[1], edge_points[2]);
                let edge_area = triangle_area(edge_points[0], edge_points[1], edge_points[2]);
                edges.insert(key, (vl, vr, edge_normal, edge_area));
            }
        }
    }

    let data: Vec<f64> = (0..n)
        .map(|i| {
            if num_neighbors[i] > 0 {
                0.5 * curvature[i] / num_neighbors[i] as f64
            } else {
                0.0
            }
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Mean_Curvature",
            data.clone(),
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            data,
            1,
        )));
    result.point_data_mut().set_active_scalars("Mean_Curvature");
    result
}

fn cross_mag(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(cross(a, b))
}

fn triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    0.5 * cross_mag(sub(b, a), sub(c, a))
}

fn triangle_angles(p: [[f64; 3]; 3]) -> [f64; 3] {
    [
        angle_between(sub(p[1], p[0]), sub(p[2], p[0])),
        angle_between(sub(p[2], p[1]), sub(p[0], p[1])),
        angle_between(sub(p[0], p[2]), sub(p[1], p[2])),
    ]
}

fn triangle_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let mut normal = cross(sub(b, a), sub(c, a));
    normalize(&mut normal);
    normal
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: &mut [f64; 3]) -> f64 {
    let len = length(*v);
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
    len
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let la = length(a);
    let lb = length(b);
    if la <= 0.0 || lb <= 0.0 {
        return 0.0;
    }
    (dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos()
}

fn valid_cell_prefix(cell: &[i64], num_points: usize) -> Option<[usize; 3]> {
    Some([
        valid_point_id(cell[0], num_points)?,
        valid_point_id(cell[1], num_points)?,
        valid_point_id(cell[2], num_points)?,
    ])
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gaussian() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = gaussian_curvature(&mesh);
        assert!(r.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn test_mean() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = mean_curvature(&mesh);
        assert!(r.point_data().get_array("MeanCurvature").is_some());
    }

    #[test]
    fn gaussian_uses_polygon_prefix_like_vtk() {
        let mesh = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3]],
        );
        let r = gaussian_curvature(&mesh);
        let arr = r.point_data().get_array("Gauss_Curvature").unwrap();
        assert_eq!(arr.num_tuples(), 4);
    }

    #[test]
    fn invalid_point_ids_are_skipped() {
        let mesh = PolyData::from_polygons(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![vec![0, 1, 2], vec![0, -1, 2], vec![0, 99, 2]],
        );
        let g = gaussian_curvature(&mesh);
        let m = mean_curvature(&mesh);
        assert_eq!(
            g.point_data()
                .get_array("Gauss_Curvature")
                .unwrap()
                .num_tuples(),
            3
        );
        assert_eq!(
            m.point_data()
                .get_array("Mean_Curvature")
                .unwrap()
                .num_tuples(),
            3
        );
    }
}
