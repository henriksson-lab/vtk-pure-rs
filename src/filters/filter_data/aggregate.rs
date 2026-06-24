use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Compute aggregate statistics of a point data scalar array.
///
/// Returns a Table with columns: "Statistic" (name), "Value" (result).
/// Computes: count, min, max, mean, sum, variance, std_dev.
pub fn aggregate(input: &PolyData, array_name: &str) -> Table {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return Table::new(),
    };

    let n = arr.num_tuples();
    if n == 0 {
        return Table::new();
    }

    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v = buf[0];
        min_v = min_v.min(v);
        max_v = max_v.max(v);
        sum += v;
        sum_sq += v * v;
    }

    let mean = sum / n as f64;
    let variance = sum_sq / n as f64 - mean * mean;
    let std_dev = variance.max(0.0).sqrt();

    let values = vec![n as f64, min_v, max_v, mean, sum, variance, std_dev];
    let _names = vec!["count", "min", "max", "mean", "sum", "variance", "std_dev"];

    // Store as parallel columns
    let mut table = Table::new();
    table.add_column(AnyDataArray::F64(DataArray::from_vec("Value", values, 1)));
    table
}

/// Convenience: get min/max/mean as a tuple.
pub fn quick_stats(input: &PolyData, array_name: &str) -> Option<(f64, f64, f64)> {
    let arr = input.point_data().get_array(array_name)?;
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let mut sum = 0.0;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
        sum += buf[0];
    }

    Some((min_v, max_v, sum / n as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pd() -> PolyData {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));
        pd
    }

    #[test]
    fn aggregate_stats() {
        let pd = make_pd();
        let table = aggregate(&pd, "val");
        let values = table.column_by_name("Value").unwrap();
        let mut buf = [0.0f64];
        // count=5
        values.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
        // min=1
        values.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        // max=5
        values.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 5.0);
        // mean=3
        values.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 3.0);
        // sum=15
        values.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 15.0);
    }

    #[test]
    fn quick_stats_test() {
        let pd = make_pd();
        let (min, max, mean) = quick_stats(&pd, "val").unwrap();
        assert_eq!(min, 1.0);
        assert_eq!(max, 5.0);
        assert_eq!(mean, 3.0);
    }

    #[test]
    fn missing_array() {
        let pd = make_pd();
        assert!(quick_stats(&pd, "nope").is_none());
    }
}
