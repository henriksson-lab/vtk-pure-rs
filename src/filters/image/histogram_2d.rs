//! 2D joint histogram between two image arrays.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute 2D joint histogram between two scalar arrays.
/// Returns a bins_x * bins_y image where each pixel is the count.
pub fn joint_histogram(
    input: &ImageData,
    array_a: &str,
    array_b: &str,
    bins_x: usize,
    bins_y: usize,
) -> ImageData {
    let arr_a = match input.point_data().get_array(array_a) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::with_dimensions(1, 1, 1),
    };
    let arr_b = match input.point_data().get_array(array_b) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::with_dimensions(1, 1, 1),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut buf = [0.0f64];
    let va: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let vb: Vec<f64> = (0..n)
        .map(|i| {
            arr_b.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let min_a = va.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_a = va.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_b = vb.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_b = vb.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range_a = if (max_a - min_a).abs() < 1e-15 {
        1.0
    } else {
        max_a - min_a
    };
    let range_b = if (max_b - min_b).abs() < 1e-15 {
        1.0
    } else {
        max_b - min_b
    };

    let mut hist = vec![0.0f64; bins_x * bins_y];
    for i in 0..n {
        let bx = (((va[i] - min_a) / range_a * bins_x as f64).floor() as usize).min(bins_x - 1);
        let by = (((vb[i] - min_b) / range_b * bins_y as f64).floor() as usize).min(bins_y - 1);
        hist[bx + by * bins_x] += 1.0;
    }

    ImageData::with_dimensions(bins_x, bins_y, 1)
        .with_spacing([range_a / bins_x as f64, range_b / bins_y as f64, 1.0])
        .with_origin([min_a, min_b, 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Histogram2D",
            hist,
            1,
        )))
}

/// Compute mutual information between two arrays.
pub fn mutual_information(input: &ImageData, array_a: &str, array_b: &str, bins: usize) -> f64 {
    let hist = joint_histogram(input, array_a, array_b, bins, bins);
    let arr = hist.point_data().get_array("Histogram2D").unwrap();
    let n2 = bins * bins;
    let mut buf = [0.0f64];
    let counts: Vec<f64> = (0..n2)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let total: f64 = counts.iter().sum();
    if total < 1.0 {
        return 0.0;
    }

    let mut row_sums = vec![0.0; bins];
    let mut col_sums = vec![0.0; bins];
    for by in 0..bins {
        for bx in 0..bins {
            row_sums[by] += counts[bx + by * bins];
            col_sums[bx] += counts[bx + by * bins];
        }
    }

    let mut mi = 0.0;
    for by in 0..bins {
        for bx in 0..bins {
            let pxy = counts[bx + by * bins] / total;
            let px = col_sums[bx] / total;
            let py = row_sums[by] / total;
            if pxy > 1e-15 && px > 1e-15 && py > 1e-15 {
                mi += pxy * (pxy / (px * py)).ln();
            }
        }
    }
    mi
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_joint_hist() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        )
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "b",
            (0..100).map(|i| (i % 10) as f64).collect(),
            1,
        )));
        let h = joint_histogram(&img, "a", "b", 5, 5);
        assert_eq!(h.dimensions(), [5, 5, 1]);
    }
    #[test]
    fn test_mi_identical() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let img = ImageData::with_dimensions(10, 10, 1)
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("a", data.clone(), 1)))
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("b", data, 1)));
        let mi = mutual_information(&img, "a", "b", 10);
        assert!(mi > 0.0); // identical signals have high MI
    }
}
