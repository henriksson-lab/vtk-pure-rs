use crate::data::ImageData;

/// Compute Pearson correlation coefficient between two ImageData arrays.
///
/// Returns r in [-1, 1]. Perfect positive correlation = 1, negative = -1.
pub fn image_correlation(a: &ImageData, b: &ImageData, a_scalars: &str, b_scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(a_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(b_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    if n < 2 {
        return 0.0;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_ab = 0.0;
    let mut sum_a2 = 0.0;
    let mut sum_b2 = 0.0;

    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        sum_a += buf_a[0];
        sum_b += buf_b[0];
        sum_ab += buf_a[0] * buf_b[0];
        sum_a2 += buf_a[0] * buf_a[0];
        sum_b2 += buf_b[0] * buf_b[0];
    }

    let nf = n as f64;
    let num = nf * sum_ab - sum_a * sum_b;
    let den = ((nf * sum_a2 - sum_a * sum_a) * (nf * sum_b2 - sum_b * sum_b)).sqrt();
    if den > 1e-15 {
        num / den
    } else {
        0.0
    }
}

/// Compute covariance between two ImageData arrays.
pub fn image_covariance(a: &ImageData, b: &ImageData, a_scalars: &str, b_scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(a_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(b_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    if n < 2 {
        return 0.0;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_ab = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        sum_a += buf_a[0];
        sum_b += buf_b[0];
        sum_ab += buf_a[0] * buf_b[0];
    }
    let nf = n as f64;
    sum_ab / nf - (sum_a / nf) * (sum_b / nf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn perfect_correlation() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![2.0, 4.0, 6.0, 8.0, 10.0],
                1,
            )));

        let r = image_correlation(&img, &img, "a", "b");
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn negative_correlation() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![5.0, 4.0, 3.0, 2.0, 1.0],
                1,
            )));

        let r = image_correlation(&img, &img, "a", "b");
        assert!((r + 1.0).abs() < 1e-10);
    }

    #[test]
    fn covariance_basic() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let cov = image_covariance(&img, &img, "a", "a");
        assert!(cov > 0.0); // positive auto-covariance
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert_eq!(image_correlation(&img, &img, "nope", "nope"), 0.0);
    }
}
