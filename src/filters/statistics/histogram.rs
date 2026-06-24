use crate::data::{AnyDataArray, DataArray, Table};

/// Compute a histogram of scalar values from a data array.
///
/// Returns a `Table` with columns "BinEdges" (n_bins+1 edge values),
/// "BinCenters" (n_bins center values), and "Counts" (n_bins counts).
pub fn histogram(array: &AnyDataArray, n_bins: usize) -> Table {
    let n_bins = n_bins.max(1);

    // Read all scalar values
    let n = array.num_tuples();
    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        array.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    if values.is_empty() {
        return Table::new();
    }

    let min_val = values.iter().copied().fold(f64::MAX, f64::min);
    let max_val = values.iter().copied().fold(f64::MIN, f64::max);

    let range = (max_val - min_val).max(1e-15);
    let bin_width = range / n_bins as f64;

    // Bin edges
    let edges: Vec<f64> = (0..=n_bins)
        .map(|i| min_val + i as f64 * bin_width)
        .collect();

    // Bin centers
    let centers: Vec<f64> = (0..n_bins)
        .map(|i| min_val + (i as f64 + 0.5) * bin_width)
        .collect();

    // Count values per bin
    let mut counts = vec![0.0f64; n_bins];
    for &v in &values {
        let bin = ((v - min_val) / bin_width).floor() as usize;
        let bin = bin.min(n_bins - 1); // clamp last edge
        counts[bin] += 1.0;
    }

    // Store BinMin and BinMax per bin (same length as Counts)
    let bin_min: Vec<f64> = (0..n_bins).map(|i| edges[i]).collect();
    let bin_max: Vec<f64> = (0..n_bins).map(|i| edges[i + 1]).collect();

    let mut table = Table::new();
    table.add_column(AnyDataArray::F64(DataArray::from_vec("BinMin", bin_min, 1)));
    table.add_column(AnyDataArray::F64(DataArray::from_vec("BinMax", bin_max, 1)));
    table.add_column(AnyDataArray::F64(DataArray::from_vec(
        "BinCenters",
        centers,
        1,
    )));
    table.add_column(AnyDataArray::F64(DataArray::from_vec("Counts", counts, 1)));
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_distribution() {
        let data = AnyDataArray::F64(DataArray::from_vec(
            "values",
            (0..100).map(|i| i as f64).collect(),
            1,
        ));
        let result = histogram(&data, 10);
        let counts = result.column_by_name("Counts").unwrap();
        // Each bin should have ~10 values
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..10 {
            counts.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 5.0 && buf[0] <= 15.0);
            total += buf[0];
        }
        assert_eq!(total, 100.0);
    }

    #[test]
    fn single_value() {
        let data = AnyDataArray::F64(DataArray::from_vec("values", vec![5.0; 20], 1));
        let result = histogram(&data, 5);
        let counts = result.column_by_name("Counts").unwrap();
        // All values in one bin
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..5 {
            counts.tuple_as_f64(i, &mut buf);
            total += buf[0];
        }
        assert_eq!(total, 20.0);
    }

    #[test]
    fn bin_edges_correct() {
        let data = AnyDataArray::F64(DataArray::from_vec("values", vec![0.0, 1.0, 2.0, 3.0], 1));
        let result = histogram(&data, 3);
        let bin_min = result.column_by_name("BinMin").unwrap();
        let bin_max = result.column_by_name("BinMax").unwrap();
        let mut buf = [0.0f64];
        bin_min.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        bin_max.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }
}
