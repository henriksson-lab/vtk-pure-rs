//! Histogram equalization of a scalar point data array.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn histogram_equalize_scalar(mesh: &PolyData, array_name: &str, bins: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let bins = bins.max(2);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let mut hist = vec![0usize; bins];
    for &v in &vals {
        let bi = (((v - mn) / range * bins as f64).floor() as usize).min(bins - 1);
        hist[bi] += 1;
    }
    let mut cdf = vec![0.0f64; bins];
    cdf[0] = hist[0] as f64;
    for i in 1..bins {
        cdf[i] = cdf[i - 1] + hist[i] as f64;
    }
    let cdf_min = cdf.iter().cloned().find(|&c| c > 0.0).unwrap_or(0.0);
    let nf = n as f64;
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let bi = (((v - mn) / range * bins as f64).floor() as usize).min(bins - 1);
            (cdf[bi] - cdf_min) / (nf - cdf_min).max(1.0)
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.0, 10.0, 10.0],
                1,
            )));
        let r = histogram_equalize_scalar(&m, "s", 10);
        assert!(r.point_data().get_array("s").is_some());
    }
}
