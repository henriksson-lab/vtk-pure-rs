//! Transfer scalar data from one mesh to another by nearest vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn transfer_scalars_nearest(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
) -> PolyData {
    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return target.clone(),
    };
    let sn = source.points.len();
    let tn = target.points.len();
    let mut buf = [0.0f64];
    let svals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let spts: Vec<[f64; 3]> = (0..sn).map(|i| source.points.get(i)).collect();
    let data: Vec<f64> = (0..tn)
        .map(|i| {
            let p = target.points.get(i);
            let mut best = 0;
            let mut bd = f64::INFINITY;
            for (j, s) in spts.iter().enumerate() {
                let d = (p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2);
                if d < bd {
                    bd = d;
                    best = j;
                }
            }
            if best < svals.len() {
                svals[best]
            } else {
                0.0
            }
        })
        .collect();
    let mut r = target.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
pub fn transfer_scalars_idw(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    power: f64,
) -> PolyData {
    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return target.clone(),
    };
    let sn = source.points.len();
    let tn = target.points.len();
    let mut buf = [0.0f64];
    let svals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let data: Vec<f64> = (0..tn)
        .map(|i| {
            let p = target.points.get(i);
            let mut wsum = 0.0;
            let mut vsum = 0.0;
            for j in 0..sn {
                let s = source.points.get(j);
                let d =
                    ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2)).sqrt();
                if d < 1e-15 {
                    return svals[j];
                }
                let w = 1.0 / d.powf(power);
                wsum += w;
                vsum += w * svals[j];
            }
            if wsum > 1e-15 {
                vsum / wsum
            } else {
                0.0
            }
        })
        .collect();
    let mut r = target.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nearest() {
        let mut src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let tgt = PolyData::from_triangles(
            vec![[0.1, 0.1, 0.0], [0.9, 0.1, 0.0], [0.5, 0.9, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = transfer_scalars_nearest(&src, &tgt, "s");
        let arr = r.point_data().get_array("s").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 10.0).abs() < 1e-10);
    }
    #[test]
    fn test_idw() {
        let mut src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 10.0, 5.0],
                1,
            )));
        let tgt = PolyData::from_triangles(vec![[1.0, 0.0, 0.0]], vec![]);
        let r = transfer_scalars_idw(&src, &tgt, "s", 2.0);
        assert!(r.point_data().get_array("s").is_some());
    }
}
