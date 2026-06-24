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
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut out = Vec::with_capacity(np);
    for pi in 0..np {
        let q = probe.points.get(pi);
        let mut best_val = 0.0;
        let mut best_d2 = f64::MAX;
        for cell in &all_cells {
            if cell.len() < 3 {
                continue;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let cx = (a[0] + b[0] + c[0]) / 3.0;
            let cy = (a[1] + b[1] + c[1]) / 3.0;
            let cz = (a[2] + b[2] + c[2]) / 3.0;
            let d2 = (q[0] - cx).powi(2) + (q[1] - cy).powi(2) + (q[2] - cz).powi(2);
            if d2 < best_d2 {
                best_d2 = d2;
                let (u, v, w) = barycentric(q, a, b, c);
                let u = u.clamp(0.0, 1.0);
                let v = v.clamp(0.0, 1.0);
                let w = w.clamp(0.0, 1.0);
                let sum = u + v + w;
                if sum > 1e-15 {
                    best_val = (u * vals[cell[0] as usize]
                        + v * vals[cell[1] as usize]
                        + w * vals[cell[2] as usize])
                        / sum;
                }
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
    let src: Vec<[f64; 3]> = (0..ns).map(|i| mesh.points.get(i)).collect();

    let mut out = Vec::with_capacity(np);
    for pi in 0..np {
        let q = probe.points.get(pi);
        let mut sum_wv = 0.0;
        let mut sum_w = 0.0;
        for si in 0..ns {
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
            sum_wv += w * vals[si.min(vals.len() - 1)];
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
}
