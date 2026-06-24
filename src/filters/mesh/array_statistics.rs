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
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = values[0];
    let max = values[n - 1];
    let sum: f64 = values.iter().sum();
    let mean = sum / n as f64;
    let var: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = var.sqrt();
    let median = if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) * 0.5
    };
    let p25 = values[(n as f64 * 0.25) as usize];
    let p75 = values[((n as f64 * 0.75) as usize).min(n - 1)];
    let iqr = p75 - p25;

    let skew = if std_dev > 1e-15 {
        values
            .iter()
            .map(|v| ((v - mean) / std_dev).powi(3))
            .sum::<f64>()
            / n as f64
    } else {
        0.0
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn stats_basic() {
        let mut pd = PolyData::new();
        for i in 0..5 {
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
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert!(array_statistics(&pd, "nope").is_none());
    }
}
