//! Compute statistics of point/cell data arrays.
use crate::data::PolyData;
pub struct ArrayStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub count: usize,
}
pub fn point_array_stats(mesh: &PolyData, array_name: &str) -> Option<ArrayStats> {
    let arr = mesh.point_data().get_array(array_name)?;
    if arr.num_components() != 1 {
        return None;
    }
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    Some(compute_stats(&mut vals))
}
pub fn cell_array_stats(mesh: &PolyData, array_name: &str) -> Option<ArrayStats> {
    let arr = mesh.cell_data().get_array(array_name)?;
    if arr.num_components() != 1 {
        return None;
    }
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    Some(compute_stats(&mut vals))
}
fn compute_stats(vals: &mut Vec<f64>) -> ArrayStats {
    let n = vals.len();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / n as f64;
    let var = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if n % 2 == 0 {
        (vals[n / 2 - 1] + vals[n / 2]) / 2.0
    } else {
        vals[n / 2]
    };
    ArrayStats {
        min: mn,
        max: mx,
        mean,
        std_dev: var.sqrt(),
        median,
        count: n,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let s = point_array_stats(&m, "s").unwrap();
        assert_eq!(s.count, 3);
        assert!((s.mean - 2.0).abs() < 1e-10);
        assert!((s.median - 2.0).abs() < 1e-10);
        assert!((s.min - 1.0).abs() < 1e-10);
        assert!((s.max - 3.0).abs() < 1e-10);
    }
}
