//! Advanced image statistics: mutual information, joint histogram, entropy.

use crate::data::{AnyDataArray, DataArray, ImageData, Table};

/// Mutual information between two scalar arrays on the same ImageData.
///
/// Single implementation lives in [`crate::filters::image::histogram_2d`].
pub use crate::filters::image::histogram_2d::mutual_information;

/// Compute Shannon entropy of a scalar array.
pub fn scalar_entropy(image: &ImageData, array_name: &str, n_bins: usize) -> f64 {
    if n_bins == 0 {
        return 0.0;
    }
    let arr = match image.point_data().get_array(array_name) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = arr.num_tuples();
    if n == 0 {
        return 0.0;
    }
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let mut counts = vec![0usize; n_bins];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = (((buf[0] - min_v) / range * n_bins as f64) as usize).min(n_bins - 1);
        counts[bin] += 1;
    }
    let mut entropy = 0.0;
    for &c in &counts {
        let p = c as f64 / n as f64;
        if p > 1e-15 {
            entropy -= p * p.ln();
        }
    }
    entropy
}

/// Joint histogram of two scalar arrays as a `Table` with one row per bin pair
/// (bin centre of `array_a`, bin centre of `array_b`, count).
///
/// Thin wrapper: the histogram itself is computed by
/// [`crate::filters::image::histogram_2d::joint_histogram`], which returns it as
/// an `ImageData`; this only reshapes that image into table columns. Returns an
/// empty table when either array is missing or not single-component.
pub fn joint_histogram(image: &ImageData, array_a: &str, array_b: &str, n_bins: usize) -> Table {
    if n_bins == 0 {
        return Table::new();
    }
    let samples = match (
        image.point_data().get_array(array_a),
        image.point_data().get_array(array_b),
    ) {
        (Some(a), Some(b)) => a.num_tuples().min(b.num_tuples()),
        _ => 0,
    };
    if samples == 0 {
        return Table::new();
    }
    let histogram = crate::filters::image::histogram_2d::joint_histogram(
        image, array_a, array_b, n_bins, n_bins,
    );
    let counts = match histogram.point_data().get_array("Histogram2D") {
        Some(array) if array.num_tuples() == n_bins * n_bins => array,
        _ => return Table::new(),
    };

    let [a_min, b_min, _] = histogram.origin();
    let [bw_a, bw_b, _] = histogram.spacing();

    let mut a_data = Vec::with_capacity(n_bins * n_bins);
    let mut b_data = Vec::with_capacity(n_bins * n_bins);
    let mut c_data = Vec::with_capacity(n_bins * n_bins);
    let mut count = [0.0f64];
    for i in 0..n_bins {
        for j in 0..n_bins {
            a_data.push(a_min + (i as f64 + 0.5) * bw_a);
            b_data.push(b_min + (j as f64 + 0.5) * bw_b);
            // Row-major image: the `array_a` bin is the fast axis.
            counts.tuple_as_f64(i + j * n_bins, &mut count);
            c_data.push(count[0]);
        }
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(array_a, a_data, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(array_b, b_data, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Count", c_data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mi_identical() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        );
        // Duplicate array as "b"
        let a = img.point_data().get_array("a").unwrap();
        let mut vals = Vec::new();
        let mut buf = [0.0f64];
        for i in 0..a.num_tuples() {
            a.tuple_as_f64(i, &mut buf);
            vals.push(buf[0]);
        }
        let mut img2 = img.clone();
        img2.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("b", vals, 1)));
        let mi = mutual_information(&img2, "a", "b", 10);
        assert!(mi > 0.0, "identical arrays should have positive MI");
    }
    #[test]
    fn entropy_uniform() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let e = scalar_entropy(&img, "v", 10);
        assert!(e > 0.0);
    }
    #[test]
    fn joint_hist() {
        let mut img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        );
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                (0..100).map(|i| i as f64).collect(),
                1,
            )));
        let jh = joint_histogram(&img, "a", "b", 5);
        assert_eq!(jh.num_rows(), 25);
    }
}
