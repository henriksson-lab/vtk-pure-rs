use crate::data::{AnyDataArray, DataArray, Table};

/// Compute a histogram of scalar values from a data array.
///
/// Returns a `Table` with VTK-style "bin_extents" (bin centers) and
/// "bin_values" (bin counts) columns. Compatibility aliases for the previous
/// Rust API are also included.
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

    let mut min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if min_val == max_val {
        min_val -= 0.5;
        max_val += 0.5;
    }

    let range = max_val - min_val;
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
        let bin = ((v - min_val) / bin_width) as usize;
        let bin = bin.min(n_bins - 1); // clamp last edge
        counts[bin] += 1.0;
    }

    // Store BinMin and BinMax per bin (same length as Counts)
    let bin_min: Vec<f64> = (0..n_bins).map(|i| edges[i]).collect();
    let bin_max: Vec<f64> = (0..n_bins).map(|i| edges[i + 1]).collect();

    let mut table = Table::new();
    table.add_column(AnyDataArray::F64(DataArray::from_vec(
        "bin_extents",
        centers.clone(),
        1,
    )));
    table.add_column(AnyDataArray::F64(DataArray::from_vec(
        "bin_values",
        counts.clone(),
        1,
    )));
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
        let counts = result.column_by_name("bin_values").unwrap();
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
        let counts = result.column_by_name("bin_values").unwrap();
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

    #[test]
    fn matches_vtk_extract_histogram_sample() {
        let values = vec![
            73.0, 8.0, 67.0, 84.0, 28.0, 75.0, 20.0, 75.0, 38.0, 38.0, 39.0, 94.0, 58.0, 89.0,
            91.0, 3.0, 91.0, 76.0, 18.0, 70.0, 18.0, 69.0, 87.0, 25.0, 81.0, 24.0, 6.0, 81.0, 67.0,
            98.0, 9.0, 24.0, 40.0, 13.0, 30.0, 93.0, 46.0, 65.0, 67.0, 55.0, 56.0, 74.0, 48.0,
            28.0, 28.0, 13.0, 21.0, 33.0, 98.0, 20.0, 84.0, 69.0, 40.0, 2.0, 41.0, 70.0, 20.0,
            71.0, 14.0, 35.0, 68.0, 47.0, 59.0, 86.0, 41.0, 53.0, 57.0, 55.0, 26.0, 47.0, 44.0,
            89.0, 46.0, 35.0, 34.0, 20.0, 10.0, 77.0, 55.0, 28.0, 33.0, 70.0, 30.0, 10.0, 9.0,
            34.0, 10.0, 77.0, 39.0, 35.0, 4.0, 20.0, 53.0, 44.0, 1.0, 60.0, 77.0, 80.0, 39.0, 14.0,
        ];
        let data = AnyDataArray::F64(DataArray::from_vec("samples", values, 1));
        let result = histogram(&data, 10);
        let counts = result.column_by_name("bin_values").unwrap();
        let expected = [11.0, 11.0, 11.0, 12.0, 11.0, 9.0, 6.0, 14.0, 7.0, 8.0];
        let mut buf = [0.0f64];
        for (i, &expected_count) in expected.iter().enumerate() {
            counts.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], expected_count);
        }
    }
}
