use crate::data::PolyData;

/// Comprehensive statistics for a point data scalar array.
#[derive(Debug, Clone)]
pub struct ArrayStatistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub sum: f64,
    pub range: f64,
    pub percentile_25: f64,
    pub percentile_75: f64,
    pub iqr: f64,
    pub skewness: f64,
}

/// Compute comprehensive statistics for a point data scalar array.
pub fn array_statistics(input: &PolyData, array_name: &str) -> Option<ArrayStatistics> {
    let arr = input.point_data().get_array(array_name)?;
    if arr.num_components() != 1 {
        return None;
    }
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    values.sort_by(|a, b| a.total_cmp(b));

    let min = values[0];
    let max = values[n - 1];
    let sum: f64 = values.iter().sum();
    let mean = sum / n as f64;
    let m2: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();
    let var = if n > 1 { m2 / (n - 1) as f64 } else { 0.0 };
    let std_dev = var.sqrt();
    let median = if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) * 0.5
    };
    let p25 = percentile_sorted(&values, 0.25);
    let p75 = percentile_sorted(&values, 0.75);
    let iqr = p75 - p25;

    let near_constant = m2 * m2 <= f32::EPSILON as f64 * mean.abs();
    let skew = if near_constant || n <= 2 {
        f64::NAN
    } else {
        let m3: f64 = values.iter().map(|v| (v - mean).powi(3)).sum();
        n as f64 * m3 / ((n - 1) as f64 * (n - 2) as f64 * var * std_dev)
    };

    Some(ArrayStatistics {
        count: n,
        min,
        max,
        mean,
        median,
        variance: var,
        std_dev,
        sum,
        range: max - min,
        percentile_25: p25,
        percentile_75: p75,
        iqr,
        skewness: skew,
    })
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }

    let idx = p.clamp(0.0, 1.0) * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;

    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn stats_basic() {
        let mut pd = PolyData::new();
        for _ in 0..5 {
            pd.points.push([0.0; 3]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));

        let s = array_statistics(&pd, "v").unwrap();
        assert_eq!(s.count, 5);
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 5.0);
        assert_eq!(s.mean, 3.0);
        assert_eq!(s.median, 3.0);
        assert_eq!(s.sum, 15.0);
    }

    #[test]
    fn stats_range() {
        let mut pd = PolyData::new();
        for _ in 0..4 {
            pd.points.push([0.0; 3]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));

        let s = array_statistics(&pd, "v").unwrap();
        assert_eq!(s.range, 30.0);
        assert!((s.variance - 166.66666666666666).abs() < 1e-10);
        assert!((s.percentile_25 - 17.5).abs() < 1e-12);
        assert!((s.percentile_75 - 32.5).abs() < 1e-12);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert!(array_statistics(&pd, "nope").is_none());
    }

    #[test]
    fn vector_array_is_not_scalar_statistics() {
        let mut pd = PolyData::new();
        pd.points.push([0.0; 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vectors",
                vec![1.0, 2.0, 3.0],
                3,
            )));

        assert!(array_statistics(&pd, "vectors").is_none());
    }

    #[test]
    fn single_value_matches_descriptive_statistics_degenerate_values() {
        let mut pd = PolyData::new();
        pd.points.push([0.0; 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![7.0], 1)));

        let s = array_statistics(&pd, "v").unwrap();
        assert_eq!(s.variance, 0.0);
        assert_eq!(s.std_dev, 0.0);
        assert!(s.skewness.is_nan());
    }
}
