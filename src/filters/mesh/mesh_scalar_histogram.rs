//! Compute histogram of a scalar field on mesh vertices.
use crate::data::PolyData;

pub struct ScalarHistogram {
    pub bins: Vec<usize>,
    pub bin_edges: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

pub fn scalar_histogram(
    mesh: &PolyData,
    scalar_name: &str,
    n_bins: usize,
) -> Option<ScalarHistogram> {
    let n = mesh.points.len();
    let arr = mesh.point_data().get_array(scalar_name)?;
    let nb = n_bins.max(1);
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    if vals.is_empty() {
        return None;
    }
    let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let variance = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
    let std_dev = variance.sqrt();
    let range = (max - min).max(1e-15);
    let bin_edges: Vec<f64> = (0..=nb)
        .map(|i| min + range * i as f64 / nb as f64)
        .collect();
    let mut bins = vec![0usize; nb];
    for &v in &vals {
        let idx = ((v - min) / range * nb as f64).floor() as usize;
        bins[idx.min(nb - 1)] += 1;
    }
    Some(ScalarHistogram {
        bins,
        bin_edges,
        min,
        max,
        mean,
        std_dev,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_hist() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let h = scalar_histogram(&mesh, "f", 3).unwrap();
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
        assert!((h.mean - 2.0).abs() < 1e-9);
    }
}
