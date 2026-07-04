//! Transfer scalar data using barycentric interpolation on closest triangle.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn transfer_barycentric(source: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return target.clone(),
    };
    if arr.num_tuples() != source.points.len() {
        return target.clone();
    }
    let tn = target.points.len();
    let mut buf = [0.0f64];
    let cells: Vec<Vec<i64>> = source
        .polys
        .iter()
        .filter(|c| {
            c.len() == 3
                && c.iter()
                    .all(|&point_id| point_id >= 0 && (point_id as usize) < source.points.len())
        })
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
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let area = dot(ab, ab) * dot(ac, ac) - dot(ab, ac) * dot(ab, ac);
    if area.abs() < 1e-30 {
        return closest_vertex_bary(p, a, b, c);
    }
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    let (uc, vc, wc) = if d1 <= 0.0 && d2 <= 0.0 {
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
                            let denom = 1.0 / (va + vb + vc);
                            let v = vb * denom;
                            let w = vc * denom;
                            (1.0 - v - w, v, w)
                        }
                    }
                }
            }
        }
    };
    let proj = [
        a[0] * uc + b[0] * vc + c[0] * wc,
        a[1] * uc + b[1] * vc + c[1] * wc,
        a[2] * uc + b[2] * vc + c[2] * wc,
    ];
    let d = ((p[0] - proj[0]).powi(2) + (p[1] - proj[1]).powi(2) + (p[2] - proj[2]).powi(2)).sqrt();
    (uc, vc, wc, d)
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn closest_vertex_bary(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> (f64, f64, f64, f64) {
    let da = dot(sub(p, a), sub(p, a));
    let db = dot(sub(p, b), sub(p, b));
    let dc = dot(sub(p, c), sub(p, c));
    if da <= db && da <= dc {
        (1.0, 0.0, 0.0, da.sqrt())
    } else if db <= dc {
        (0.0, 1.0, 0.0, db.sqrt())
    } else {
        (0.0, 0.0, 1.0, dc.sqrt())
    }
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
