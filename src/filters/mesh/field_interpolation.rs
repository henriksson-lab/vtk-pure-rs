//! Field interpolation on meshes: barycentric, nearest, IDW.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Interpolate a scalar field at arbitrary probe points using barycentric
/// coordinates on the nearest triangle.
pub fn barycentric_interpolate(mesh: &PolyData, array_name: &str, probe: &PolyData) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return probe.clone(),
    };
    let np = probe.points.len();
    let triangles = surface_triangles(mesh);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    if vals.is_empty() {
        return probe.clone();
    }

    let mut out = Vec::with_capacity(np);
    for pi in 0..np {
        let q = probe.points.get(pi);
        let mut best_val = 0.0;
        let mut best_d2 = f64::MAX;
        for &[id0, id1, id2] in &triangles {
            let Some(i0) = valid_data_point_id(id0, mesh.points.len(), vals.len()) else {
                continue;
            };
            let Some(i1) = valid_data_point_id(id1, mesh.points.len(), vals.len()) else {
                continue;
            };
            let Some(i2) = valid_data_point_id(id2, mesh.points.len(), vals.len()) else {
                continue;
            };
            let a = mesh.points.get(i0);
            let b = mesh.points.get(i1);
            let c = mesh.points.get(i2);
            let (u, v, w, d2) = closest_triangle_barycentric(q, a, b, c);
            if d2 < best_d2 {
                best_d2 = d2;
                best_val = u * vals[i0] + v * vals[i1] + w * vals[i2];
            }
        }
        out.push(best_val);
    }

    let mut result = probe.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, out, 1)));
    result
}

/// Inverse-distance-weighted interpolation from mesh vertices to probe points.
pub fn idw_interpolate(
    mesh: &PolyData,
    array_name: &str,
    probe: &PolyData,
    power: f64,
    radius: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return probe.clone(),
    };
    let ns = mesh.points.len();
    let np = probe.points.len();
    let r2 = radius * radius;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..ns.min(arr.num_tuples()))
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    if vals.is_empty() {
        return probe.clone();
    }
    let src: Vec<[f64; 3]> = (0..ns).map(|i| mesh.points.get(i)).collect();

    let mut out = Vec::with_capacity(np);
    for pi in 0..np {
        let q = probe.points.get(pi);
        let mut sum_wv = 0.0;
        let mut sum_w = 0.0;
        for si in 0..vals.len() {
            let d2 = (q[0] - src[si][0]).powi(2)
                + (q[1] - src[si][1]).powi(2)
                + (q[2] - src[si][2]).powi(2);
            if d2 > r2 {
                continue;
            }
            let w = if d2 < 1e-20 {
                1e15
            } else {
                1.0 / d2.powf(power / 2.0)
            };
            sum_wv += w * vals[si];
            sum_w += w;
        }
        out.push(if sum_w > 1e-15 { sum_wv / sum_w } else { 0.0 });
    }

    let mut result = probe.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, out, 1)));
    result
}

fn surface_triangles(mesh: &PolyData) -> Vec<[i64; 3]> {
    let mut triangles = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let base = cell[0];
        for i in 1..cell.len() - 1 {
            triangles.push([base, cell[i], cell[i + 1]]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            if i % 2 == 0 {
                triangles.push([strip[i], strip[i + 1], strip[i + 2]]);
            } else {
                triangles.push([strip[i + 1], strip[i], strip[i + 2]]);
            }
        }
    }
    triangles
}

fn barycentric(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> (f64, f64, f64) {
    let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let v2 = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d00 = v0[0] * v0[0] + v0[1] * v0[1] + v0[2] * v0[2];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1] + v0[2] * v1[2];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1] + v2[2] * v0[2];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1] + v2[2] * v1[2];
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-15 {
        return (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    (1.0 - v - w, v, w)
}

fn closest_triangle_barycentric(
    p: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> (f64, f64, f64, f64) {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    let (u, v, w) = if d1 <= 0.0 && d2 <= 0.0 {
        (1.0, 0.0, 0.0)
    } else {
        let bp = sub(p, b);
        let d3 = dot(ab, bp);
        let d4 = dot(ac, bp);
        if d3 >= 0.0 && d4 <= d3 {
            (0.0, 1.0, 0.0)
        } else {
            let vc = d1 * d4 - d3 * d2;
            if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
                let v = d1 / (d1 - d3);
                (1.0 - v, v, 0.0)
            } else {
                let cp = sub(p, c);
                let d5 = dot(ab, cp);
                let d6 = dot(ac, cp);
                if d6 >= 0.0 && d5 <= d6 {
                    (0.0, 0.0, 1.0)
                } else {
                    let vb = d5 * d2 - d1 * d6;
                    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
                        let w = d2 / (d2 - d6);
                        (1.0 - w, 0.0, w)
                    } else {
                        let va = d3 * d6 - d5 * d4;
                        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
                            let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
                            (0.0, 1.0 - w, w)
                        } else {
                            let denom = va + vb + vc;
                            if denom.abs() < 1e-15 {
                                (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
                            } else {
                                let v = vb / denom;
                                let w = vc / denom;
                                (1.0 - v - w, v, w)
                            }
                        }
                    }
                }
            }
        }
    };
    let q = [
        u * a[0] + v * b[0] + w * c[0],
        u * a[1] + v * b[1] + w * c[1],
        u * a[2] + v * b[2] + w * c[2],
    ];
    let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
    (u, v, w, d2)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn valid_data_point_id(point_id: i64, n_points: usize, n_values: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points && point_id < n_values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;
    #[test]
    fn bary_interp() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "t",
                vec![0.0, 2.0, 1.0],
                1,
            )));
        let probe = PolyData::from_points(vec![[1.0, 0.5, 0.0]]);
        let result = barycentric_interpolate(&mesh, "t", &probe);
        let arr = result.point_data().get_array("t").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0 && buf[0] < 2.0);
    }
    #[test]
    fn idw() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0],
                1,
            )));
        let probe = PolyData::from_points(vec![[0.5, 0.0, 0.0]]);
        let result = idw_interpolate(&mesh, "v", &probe, 2.0, 5.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 0.1); // midpoint
    }

    #[test]
    fn bary_interp_uses_closest_triangle_not_centroid() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [100.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.0, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "t",
                vec![1.0, 1.0, 1.0, 100.0, 100.0, 100.0],
                1,
            )));
        let probe = PolyData::from_points(vec![[1.0, 0.1, 0.0]]);
        let result = barycentric_interpolate(&mesh, "t", &probe);
        let arr = result.point_data().get_array("t").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn bary_interp_skips_ids_outside_points_even_with_extra_values() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.polys.push_cell(&[0, 1, 99]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "t",
                vec![1.0; 100],
                1,
            )));

        let probe = PolyData::from_points(vec![[0.1, 0.1, 0.0]]);
        let result = barycentric_interpolate(&mesh, "t", &probe);
        let arr = result.point_data().get_array("t").unwrap();
        let mut buf = [1.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
