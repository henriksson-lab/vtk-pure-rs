//! Sample a scalar field at query points using nearest-neighbor or IDW.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn sample_nearest(mesh: &PolyData, array_name: &str, query_points: &[[f64; 3]]) -> Vec<f64> {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return vec![0.0; query_points.len()],
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    query_points
        .iter()
        .map(|q| {
            let mut best = 0;
            let mut bd = f64::INFINITY;
            for j in 0..n {
                let p = mesh.points.get(j);
                let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2);
                if d < bd {
                    bd = d;
                    best = j;
                }
            }
            if best < vals.len() {
                vals[best]
            } else {
                0.0
            }
        })
        .collect()
}
pub fn sample_idw(
    mesh: &PolyData,
    array_name: &str,
    query_points: &[[f64; 3]],
    power: f64,
) -> Vec<f64> {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return vec![0.0; query_points.len()],
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    query_points
        .iter()
        .map(|q| {
            let mut wsum = 0.0;
            let mut vsum = 0.0;
            for j in 0..n {
                let p = mesh.points.get(j);
                let d =
                    ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt();
                if d < 1e-15 {
                    return vals[j];
                }
                let w = 1.0 / d.powf(power);
                wsum += w;
                vsum += w * vals[j];
            }
            if wsum > 1e-15 {
                vsum / wsum
            } else {
                0.0
            }
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nearest() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let r = sample_nearest(&m, "s", &[[0.1, 0.1, 0.0]]);
        assert!((r[0] - 10.0).abs() < 1e-10);
    }
    #[test]
    fn test_idw() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 10.0, 5.0],
                1,
            )));
        let r = sample_idw(&m, "s", &[[1.0, 0.0, 0.0]], 2.0);
        assert!(r[0] > 2.0 && r[0] < 8.0);
    }
}
