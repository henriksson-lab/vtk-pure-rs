//! Compute statistics (min, max, mean, std, median) of a scalar field.
use crate::data::PolyData;

pub struct ScalarStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub count: usize,
}

pub fn scalar_stats(mesh: &PolyData, scalar_name: &str) -> Option<ScalarStats> {
    let n = mesh.points.len();
    let arr = mesh.point_data().get_array(scalar_name)?;
    if n == 0 {
        return None;
    }
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / n as f64;
    let variance = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };
    Some(ScalarStats {
        min,
        max,
        mean,
        std_dev: variance.sqrt(),
        median,
        count: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_stats() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let s = scalar_stats(&mesh, "v").unwrap();
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 3.0);
        assert!((s.mean - 2.0).abs() < 1e-9);
        assert_eq!(s.median, 2.0);
    }
}
