//! Compute extrinsic curvature measures (shape operator eigenvalues).
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

pub fn shape_operator_trace(mesh: &PolyData) -> PolyData {
    let trace: Vec<f64> = mean_curvature(mesh).into_iter().map(|h| 2.0 * h).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeTrace",
            trace,
            1,
        )));
    r.point_data_mut().set_active_scalars("ShapeTrace");
    r
}

pub fn shape_operator_det(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeDet",
            gauss_curvature(mesh),
            1,
        )));
    r.point_data_mut().set_active_scalars("ShapeDet");
    r
}

fn mean_curvature(mesh: &PolyData) -> Vec<f64> {
    let n = mesh.points.len();
    let triangles = valid_triangles(mesh);
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for (face_id, tri) in triangles.iter().enumerate() {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            edge_faces
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(face_id);
        }
    }

    let mut mean_curvature_data = vec![0.0f64; n];
    let mut num_neighb = vec![0usize; n];
    for (face_id, tri) in triangles.iter().enumerate() {
        for i in 0..3 {
            let v_l = tri[i];
            let v_r = tri[(i + 1) % 3];
            let v_o = tri[(i + 2) % 3];
            let key = if v_l < v_r { (v_l, v_r) } else { (v_r, v_l) };
            let Some(faces) = edge_faces.get(&key) else {
                continue;
            };
            if faces.len() != 2 {
                continue;
            }
            let n_face = if faces[0] == face_id {
                faces[1]
            } else {
                faces[0]
            };
            if n_face <= face_id {
                continue;
            }

            let ore = mesh.points.get(v_l);
            let end = mesh.points.get(v_r);
            let oth = mesh.points.get(v_o);
            let n_f = normal(ore, end, oth);
            let mut e = [end[0] - ore[0], end[1] - ore[1], end[2] - ore[2]];
            let length = normalize(&mut e);
            let mut area = triangle_area(ore, end, oth);

            let neigh = triangles[n_face];
            let vn0 = mesh.points.get(neigh[0]);
            let vn1 = mesh.points.get(neigh[1]);
            let vn2 = mesh.points.get(neigh[2]);
            area += triangle_area(vn0, vn1, vn2);
            let n_n = normal(vn0, vn1, vn2);

            let cs = dot(n_f, n_n);
            let sn = dot(cross(n_f, n_n), e);
            let mut hf = if sn != 0.0 || cs != 0.0 {
                length * sn.atan2(cs)
            } else {
                0.0
            };
            if area != 0.0 {
                hf = 3.0 * hf / area;
            }

            mean_curvature_data[v_l] += hf;
            mean_curvature_data[v_r] += hf;
            num_neighb[v_l] += 1;
            num_neighb[v_r] += 1;
        }
    }

    mean_curvature_data
        .into_iter()
        .zip(num_neighb)
        .map(|(h, count)| {
            if count > 0 {
                0.5 * h / count as f64
            } else {
                0.0
            }
        })
        .collect()
}

fn gauss_curvature(mesh: &PolyData) -> Vec<f64> {
    let n = mesh.points.len();
    let mut k = vec![2.0 * std::f64::consts::PI; n];
    let mut d_a = vec![0.0f64; n];

    for tri in valid_triangles(mesh) {
        let v0 = mesh.points.get(tri[0]);
        let v1 = mesh.points.get(tri[1]);
        let v2 = mesh.points.get(tri[2]);
        let e0 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e1 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
        let e2 = [v0[0] - v2[0], v0[1] - v2[1], v0[2] - v2[2]];

        let alpha0 = std::f64::consts::PI - angle_between(e1, e2);
        let alpha1 = std::f64::consts::PI - angle_between(e2, e0);
        let alpha2 = std::f64::consts::PI - angle_between(e0, e1);
        let area = triangle_area(v0, v1, v2);

        d_a[tri[0]] += area;
        d_a[tri[1]] += area;
        d_a[tri[2]] += area;
        k[tri[0]] -= alpha1;
        k[tri[1]] -= alpha2;
        k[tri[2]] -= alpha0;
    }

    k.into_iter()
        .zip(d_a)
        .map(|(k, area)| if area > 0.0 { 3.0 * k / area } else { 0.0 })
        .collect()
}

fn valid_triangles(mesh: &PolyData) -> Vec<[usize; 3]> {
    let n = mesh.points.len();
    let mut triangles: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter_map(|cell| {
            if cell.len() != 3 {
                return None;
            }
            valid_triangle(cell[0], cell[1], cell[2], n)
        })
        .collect();

    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        let mut p1 = strip[0];
        let mut p2 = strip[1];
        for i in 0..strip.len() - 2 {
            let p3 = strip[i + 2];
            let tri = if i % 2 == 0 {
                valid_triangle(p1, p2, p3, n)
            } else {
                valid_triangle(p2, p1, p3, n)
            };
            if let Some(tri) = tri {
                triangles.push(tri);
            }
            p1 = p2;
            p2 = p3;
        }
    }

    triangles
}

fn valid_triangle(a: i64, b: i64, c: i64, num_points: usize) -> Option<[usize; 3]> {
    let a = usize::try_from(a).ok()?;
    let b = usize::try_from(b).ok()?;
    let c = usize::try_from(c).ok()?;
    (a < num_points && b < num_points && c < num_points).then_some([a, b, c])
}

fn normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let mut n = cross(u, v);
    normalize(&mut n);
    n
}

fn triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cr = cross(u, v);
    0.5 * (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt()
}

fn normalize(v: &mut [f64; 3]) -> f64 {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.0 {
        v[0] /= length;
        v[1] /= length;
        v[2] /= length;
    }
    length
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    if la > 0.0 && lb > 0.0 {
        (dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos()
    } else {
        0.0
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trace() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = shape_operator_trace(&m);
        assert!(r.point_data().get_array("ShapeTrace").is_some());
    }
    #[test]
    fn test_det() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = shape_operator_det(&m);
        assert!(r.point_data().get_array("ShapeDet").is_some());
    }

    #[test]
    fn test_det_from_triangle_strip() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.points.push([1.0, 1.0, 0.0]);
        m.strips.push_cell(&[0, 1, 2, 3]);

        let r = shape_operator_det(&m);
        let arr = r.point_data().get_array("ShapeDet").unwrap();
        let mut value = [0.0f64];
        arr.tuple_as_f64(0, &mut value);
        assert!(value[0] != 0.0);
    }
}
