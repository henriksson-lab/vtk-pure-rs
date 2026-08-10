//! Image histogram analysis: computation, equalization, matching, thresholds.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute a histogram with configurable bins and return bin centers + counts.
///
/// Thin adapter over the single histogram implementation in
/// [`crate::filters::image::histogram_compute::compute_histogram`]; the bin
/// centers are the midpoints of that function's bin edges. Returns empty
/// vectors when the array is missing or empty.
pub fn compute_histogram(
    image: &ImageData,
    array_name: &str,
    n_bins: usize,
) -> (Vec<f64>, Vec<usize>) {
    let Some(result) =
        crate::filters::image::histogram_compute::compute_histogram(image, array_name, n_bins)
    else {
        return (Vec::new(), Vec::new());
    };
    let centers: Vec<f64> = result
        .bin_edges
        .windows(2)
        .map(|edges| 0.5 * (edges[0] + edges[1]))
        .collect();
    (centers, result.counts)
}

/// Histogram equalization: redistribute values for uniform histogram.
pub fn histogram_equalize(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return image.clone();
    }
    let mut buf = [0.0f64];
    let mut values: Vec<(f64, usize)> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0], i)
        })
        .collect();
    values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut output = vec![0.0f64; n];
    for (rank, &(_, idx)) in values.iter().enumerate() {
        output[idx] = rank as f64 / (n - 1).max(1) as f64;
    }

    let mut result = image.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..image.point_data().num_arrays() {
        let a = image.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                output.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *result.point_data_mut() = attrs;
    result
}

/// Compute Otsu's optimal threshold for bimodal distribution.
///
/// Fixed 256-bin convenience form of
/// [`crate::filters::image::otsu::otsu_threshold`], which holds the single
/// implementation. Returns `0.0` when the array is missing or empty.
pub fn otsu_threshold(image: &ImageData, array_name: &str) -> f64 {
    crate::filters::image::otsu::otsu_threshold(image, array_name, 256).unwrap_or(0.0)
}

/// Apply a threshold determined by Otsu's method.
pub fn auto_threshold(image: &ImageData, array_name: &str) -> ImageData {
    let thresh = otsu_threshold(image, array_name);
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let n = arr.num_tuples();
    let mut output = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        output.push(if buf[0] >= thresh { 1.0 } else { 0.0 });
    }
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn histogram() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let (centers, counts) = compute_histogram(&img, "v", 5);
        assert_eq!(centers.len(), 5);
        assert_eq!(counts.iter().sum::<usize>(), 100);
    }
    #[test]
    fn equalize() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * x,
        );
        let result = histogram_equalize(&img, "v");
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn otsu() {
        // Bimodal: values clustered near 0.2 and 0.8
        let mut vals: Vec<f64> = (0..50).map(|i| 0.15 + 0.1 * (i as f64 / 50.0)).collect();
        vals.extend((0..50).map(|i| 0.75 + 0.1 * (i as f64 / 50.0)));
        let mut img = ImageData::with_dimensions(100, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));
        let t = otsu_threshold(&img, "v");
        // Should find a threshold between the two clusters
        assert!(t > 0.2 && t < 0.8, "threshold={t}");
    }
    #[test]
    fn auto_thresh() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = auto_threshold(&img, "v");
        assert!(result.point_data().get_array("v").is_some());
    }

    #[test]
    fn zero_bins_are_clamped() {
        let img = ImageData::from_function(
            [2, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let (centers, counts) = compute_histogram(&img, "v", 0);
        assert_eq!(centers.len(), 1);
        assert_eq!(counts.iter().sum::<usize>(), 2);
    }

    #[test]
    fn histogram_handles_all_negative_values() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-3.0, -2.0, -1.0], 1),
        ));
        let (centers, counts) = compute_histogram(&img, "v", 2);
        assert!(centers[0] < 0.0);
        assert_eq!(counts.iter().sum::<usize>(), 3);
    }
}
