//! Transfer scalar data using barycentric interpolation on closest triangle.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn transfer_barycentric(source: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return target.clone(),
    };
    let tn = target.points.len();
    let mut buf = [0.0f64];
    let cells: Vec<Vec<i64>> = source
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| c.to_vec())
        .collect();
    let data: Vec<f64> = (0..tn)
        .map(|i| {
            let p = target.points.get(i);
            let mut best_d = f64::INFINITY;
            let mut best_val = 0.0;
            for c in &cells {
                let a = source.points.get(c[0] as usize);
                let b = source.points.get(c[1] as usize);
                let cc = source.points.get(c[2] as usize);
                let (u, v, w, d) = closest_bary(p, a, b, cc);
                if d < best_d {
                    best_d = d;
                    arr.tuple_as_f64(c[0] as usize, &mut buf);
                    let va = buf[0];
                    arr.tuple_as_f64(c[1] as usize, &mut buf);
                    let vb = buf[0];
                    arr.tuple_as_f64(c[2] as usize, &mut buf);
                    let vc = buf[0];
                    best_val = u * va + v * vb + w * vc;
                }
            }
            best_val
        })
        .collect();
    let mut r = target.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
fn closest_bary(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> (f64, f64, f64, f64) {
    let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let v2 = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d00 = v0[0] * v0[0] + v0[1] * v0[1] + v0[2] * v0[2];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1] + v0[2] * v1[2];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1] + v2[2] * v0[2];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1] + v2[2] * v1[2];
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-30 {
        return (1.0, 0.0, 0.0, f64::INFINITY);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    let uc = u.clamp(0.0, 1.0);
    let vc = v.clamp(0.0, 1.0);
    let wc = w.clamp(0.0, 1.0);
    let s = uc + vc + wc;
    let (uc, vc, wc) = if s > 1e-15 {
        (uc / s, vc / s, wc / s)
    } else {
        (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    };
    let proj = [
        a[0] * uc + b[0] * vc + c[0] * wc,
        a[1] * uc + b[1] * vc + c[1] * wc,
        a[2] * uc + b[2] * vc + c[2] * wc,
    ];
    let d = ((p[0] - proj[0]).powi(2) + (p[1] - proj[1]).powi(2) + (p[2] - proj[2]).powi(2)).sqrt();
    (uc, vc, wc, d)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 2.0, 1.0],
                1,
            )));
        let tgt = PolyData::from_triangles(vec![[1.0, 0.5, 0.0]], vec![]);
        let r = transfer_barycentric(&src, &tgt, "s");
        assert!(r.point_data().get_array("s").is_some());
    }
}
