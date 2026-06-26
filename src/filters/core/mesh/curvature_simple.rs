//! Simple discrete curvature estimation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian curvature using angle defect.
pub fn gaussian_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut curvature = vec![2.0 * std::f64::consts::PI; n];
    let mut area = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
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
            "GaussianCurvature",
            curvature,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("GaussianCurvature");
    result
}

/// Compute discrete mean curvature using VTK-style signed dihedral edges.
pub fn mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut curvature = vec![0.0f64; n];
    let mut num_neighbors = vec![0usize; n];
    let mut edges = std::collections::HashMap::<(usize, usize), ([usize; 3], [f64; 3], f64)>::new();

    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let normal = triangle_normal(p[0], p[1], p[2]);
        let face_area = triangle_area(p[0], p[1], p[2]);

        for i in 0..3 {
            let vl = ids[i];
            let vr = ids[(i + 1) % 3];
            let key = if vl < vr { (vl, vr) } else { (vr, vl) };
            if let Some((_, other_normal, other_area)) = edges.remove(&key) {
                let start = mesh.points.get(vl);
                let end = mesh.points.get(vr);
                let mut edge = sub(end, start);
                let length = normalize(&mut edge);
                let cs = dot(normal, other_normal);
                let sn = dot(cross(normal, other_normal), edge);
                let mut hf = if sn != 0.0 || cs != 0.0 {
                    length * sn.atan2(cs)
                } else {
                    0.0
                };
                let total_area = face_area + other_area;
                if total_area != 0.0 {
                    hf = 3.0 * hf / total_area;
                }
                curvature[vl] += hf;
                curvature[vr] += hf;
                num_neighbors[vl] += 1;
                num_neighbors[vr] += 1;
            } else {
                edges.insert(key, (ids, normal, face_area));
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
            "MeanCurvature",
            data,
            1,
        )));
    result.point_data_mut().set_active_scalars("MeanCurvature");
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
}
