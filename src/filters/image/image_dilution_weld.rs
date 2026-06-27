//! Weld dilution ratio
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_dilution_weld(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let denominator = buf[0] + buf[0] * 0.3;
            if denominator.abs() > 1e-30 {
                buf[0] / denominator
            } else {
                0.0
            }
        })
        .collect();
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
        let r = image_dilution_weld(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn computes_weld_dilution_ratio_and_handles_zero() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![2.6, 0.0], 1),
        ));
        let r = image_dilution_weld(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();
        assert!((values[0] - (2.6 / (2.6 + 2.6 * 0.3))).abs() <= 1e-12);
        assert_eq!(values[1], 0.0);
    }
}
