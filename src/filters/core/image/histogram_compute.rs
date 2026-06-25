use crate::data::ImageData;

/// Result of a histogram computation.
#[derive(Debug, Clone)]
pub struct HistogramResult {
    /// Bin edges (length = num_bins + 1).
    pub bin_edges: Vec<f64>,
    /// Counts per bin (length = num_bins).
    pub counts: Vec<usize>,
    /// Total number of values processed.
    pub total: usize,
    /// Minimum value encountered.
    pub min: f64,
    /// Maximum value encountered.
    pub max: f64,
}

/// Compute a histogram of scalar values from an ImageData point data array.
///
/// Returns `None` if the named array does not exist or has zero tuples.
pub fn compute_histogram(
    input: &ImageData,
    scalars: &str,
    num_bins: usize,
) -> Option<HistogramResult> {
    let arr = input.point_data().get_array(scalars)?;
    let n: usize = arr.num_tuples();
    if n == 0 {
        return None;
    }
    let num_bins: usize = num_bins.max(1);

    // First pass: find min/max
    let mut min_v: f64 = f64::MAX;
    let mut max_v: f64 = f64::NEG_INFINITY;
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v: f64 = buf[0];
        if v < min_v {
            min_v = v;
        }
        if v > max_v {
            max_v = v;
        }
    }

    let range: f64 = (max_v - min_v).max(1e-15);
    let bin_width: f64 = range / num_bins as f64;

    // Build bin edges
    let bin_edges: Vec<f64> = (0..=num_bins)
        .map(|i| min_v + i as f64 * bin_width)
        .collect();

    // Second pass: count
    let mut counts: Vec<usize> = vec![0; num_bins];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v: f64 = buf[0];
        let bin: usize = ((v - min_v) / bin_width).floor() as usize;
        let bin: usize = bin.min(num_bins - 1);
        counts[bin] += 1;
    }

    Some(HistogramResult {
        bin_edges,
        counts,
        total: n,
        min: min_v,
        max: max_v,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};

    fn make_image_with_scalars(values: Vec<f64>) -> ImageData {
        let n: usize = values.len();
        let mut image = ImageData::with_dimensions(n, 1, 1);
        let arr = DataArray::from_vec("TestScalars", values, 1);
        image.point_data_mut().add_array(AnyDataArray::F64(arr));
        image
    }

    #[test]
    fn histogram_uniform_values() {
        let values: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let image = make_image_with_scalars(values);
        let result = compute_histogram(&image, "TestScalars", 5).unwrap();
        assert_eq!(result.counts.len(), 5);
        assert_eq!(result.bin_edges.len(), 6);
        assert_eq!(result.total, 10);
        // Each bin should have 2 values (0-1, 2-3, 4-5, 6-7, 8-9)
        let sum: usize = result.counts.iter().sum();
        assert_eq!(sum, 10);
    }

    #[test]
    fn histogram_single_value() {
        let values: Vec<f64> = vec![5.0, 5.0, 5.0];
        let image = make_image_with_scalars(values);
        let result = compute_histogram(&image, "TestScalars", 3).unwrap();
        assert_eq!(result.total, 3);
        // All values are the same, so all go into one bin
        let sum: usize = result.counts.iter().sum();
        assert_eq!(sum, 3);
    }

    #[test]
    fn histogram_missing_array_returns_none() {
        let image = ImageData::with_dimensions(2, 2, 2);
        let result = compute_histogram(&image, "NonExistent", 10);
        assert!(result.is_none());
    }

    #[test]
    fn histogram_zero_bins_clamps_to_one() {
        let values: Vec<f64> = vec![1.0, 2.0, 3.0];
        let image = make_image_with_scalars(values);
        let result = compute_histogram(&image, "TestScalars", 0).unwrap();
        assert_eq!(result.counts.len(), 1);
        assert_eq!(result.bin_edges.len(), 2);
        assert_eq!(result.total, 3);
    }

    #[test]
    fn histogram_all_negative_values() {
        let image = make_image_with_scalars(vec![-3.0, -2.0, -1.0]);
        let result = compute_histogram(&image, "TestScalars", 2).unwrap();
        assert_eq!(result.max, -1.0);
        assert_eq!(result.counts.iter().sum::<usize>(), 3);
    }
}
