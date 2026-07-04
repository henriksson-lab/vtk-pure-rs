use crate::data::{AnyDataArray, DataArray, ImageData};

/// Quantize an ImageData scalar field to N discrete levels.
///
/// Maps continuous values to the nearest of N evenly-spaced levels.
pub fn image_quantize(input: &ImageData, scalars: &str, n_levels: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return input.clone(),
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }
    let nl = n_levels.max(2);
    let mut buf = [0.0f64];

    let mut min_v = f64::MAX;
    let mut max_v = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);

    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let t = (buf[0] - min_v) / range;
            let level = (t * (nl - 1) as f64).round() as usize;
            min_v + level.min(nl - 1) as f64 * range / (nl - 1) as f64
        })
        .collect();

    let mut img = input.clone();
    let mut attrs = input.point_data().clone();
    attrs.remove_array(scalars);
    attrs.add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    attrs.set_active_scalars(scalars);
    *img.point_data_mut() = attrs;
    img
}

/// Posterize: reduce to N color levels using equal-width binning.
pub fn image_posterize(input: &ImageData, scalars: &str, n_levels: usize) -> ImageData {
    image_quantize(input, scalars, n_levels)
}

/// Dither: add noise before quantizing to reduce banding artifacts.
pub fn image_dither_quantize(
    input: &ImageData,
    scalars: &str,
    n_levels: usize,
    seed: u64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return input.clone(),
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }
    let nl = n_levels.max(2);
    let mut buf = [0.0f64];

    let mut min_v = f64::MAX;
    let mut max_v = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let step = range / (nl - 1) as f64;

    let mut rng = seed;
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((rng >> 33) as f64 / (1u64 << 31) as f64 - 0.5) * step;
            let v = buf[0] + noise;
            let t = (v - min_v) / range;
            let level = (t * (nl - 1) as f64).round() as usize;
            min_v + level.min(nl - 1) as f64 * step
        })
        .collect();

    let mut img = input.clone();
    let mut attrs = input.point_data().clone();
    attrs.remove_array(scalars);
    attrs.add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    attrs.set_active_scalars(scalars);
    *img.point_data_mut() = attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_to_2_levels() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.3, 0.5, 0.7, 1.0],
                1,
            )));

        let result = image_quantize(&img, "v", 2);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // Should be either 0.0 or 1.0
        for i in 0..5 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] == 0.0 || buf[0] == 1.0);
        }
    }

    #[test]
    fn quantize_preserves_range() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 50.0, 90.0],
                1,
            )));

        let result = image_quantize(&img, "v", 3);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 90.0);
    }

    #[test]
    fn dither_quantize() {
        let mut img = ImageData::with_dimensions(10, 1, 1);
        let vals: Vec<f64> = (0..10).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));

        let result = image_dither_quantize(&img, "v", 3, 42);
        assert!(result.point_data().get_array("v").is_some());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_quantize(&img, "nope", 5);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }

    #[test]
    fn vector_array_is_not_reinterpreted_as_scalar() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![0.0, 0.5, 1.0, 1.0, 0.5, 0.0],
                3,
            )));

        let result = image_quantize(&img, "rgb", 2);
        let arr = result.point_data().get_array("rgb").unwrap();
        assert_eq!(arr.num_components(), 3);
    }
}
