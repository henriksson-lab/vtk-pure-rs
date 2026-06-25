use crate::data::ImageData;

/// Compute Peak Signal-to-Noise Ratio between two images.
///
/// PSNR = 10*log10(MAX²/MSE) where MAX is the peak signal value.
/// Higher = more similar. Returns (psnr_db, mse).
pub fn image_psnr(a: &ImageData, b: &ImageData, scalars: &str, max_value: f64) -> (f64, f64) {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return (0.0, 0.0),
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return (0.0, 0.0),
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    let num_components = aa.num_components().min(ba.num_components());
    if n == 0 || num_components == 0 {
        return (0.0, 0.0);
    }

    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut mse = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            mse += (buf_a[c] - buf_b[c]).powi(2);
        }
    }
    mse /= (n * num_components) as f64;

    let psnr = if mse > 1e-15 {
        10.0 * (max_value * max_value / mse).log10()
    } else {
        f64::INFINITY
    };
    (psnr, mse)
}

/// Compute Mean Absolute Error between two images.
pub fn image_mae(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    let num_components = aa.num_components().min(ba.num_components());
    if n == 0 || num_components == 0 {
        return 0.0;
    }
    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut mae = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            mae += (buf_a[c] - buf_b[c]).abs();
        }
    }
    mae / (n * num_components) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn identical_infinite_psnr() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![50.0; 4],
                1,
            )));
        let (psnr, mse) = image_psnr(&img, &img, "v", 255.0);
        assert!(psnr == f64::INFINITY);
        assert_eq!(mse, 0.0);
    }

    #[test]
    fn known_mse() {
        let mut a = ImageData::with_dimensions(4, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 4], 1)));
        let mut b = ImageData::with_dimensions(4, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0; 4],
                1,
            )));

        let (_, mse) = image_psnr(&a, &b, "v", 255.0);
        assert!((mse - 100.0).abs() < 1e-10);
    }

    #[test]
    fn mae_basic() {
        let mut a = ImageData::with_dimensions(4, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 4], 1)));
        let mut b = ImageData::with_dimensions(4, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 4], 1)));

        assert!((image_mae(&a, &b, "v") - 5.0).abs() < 1e-10);
    }

    #[test]
    fn multi_component_arrays_use_all_components() {
        let mut a = ImageData::with_dimensions(1, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![0.0, 0.0, 0.0],
                3,
            )));
        let mut b = ImageData::with_dimensions(1, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![0.0, 3.0, 4.0],
                3,
            )));

        let (_, mse) = image_psnr(&a, &b, "rgb", 255.0);
        assert!((mse - 25.0 / 3.0).abs() < 1e-10);
        assert!((image_mae(&a, &b, "rgb") - 7.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let (psnr, _) = image_psnr(&img, &img, "nope", 255.0);
        assert_eq!(psnr, 0.0);
    }
}
