//! Michelson contrast
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_michelson_contrast(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut i_min = f64::INFINITY;
    let mut i_max = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        i_min = i_min.min(buf[0]);
        i_max = i_max.max(buf[0]);
    }
    let denominator = i_max + i_min;
    let contrast = if n == 0 || denominator.abs() < 1e-15 {
        0.0
    } else {
        (i_max - i_min) / denominator
    };
    let data = vec![contrast; n];
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_michelson_contrast(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0 / 3.0).abs() < 1e-12);
    }
}
