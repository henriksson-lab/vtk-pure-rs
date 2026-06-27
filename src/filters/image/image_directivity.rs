//! Antenna directivity
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_directivity(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let denominator = buf[0] * 4.0 * std::f64::consts::PI;
            if denominator.abs() > 1e-30 {
                4.0 * std::f64::consts::PI * buf[0].abs().max(0.01) / denominator
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
        let r = image_directivity(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn computes_directivity_formula_and_handles_zero() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![2.0, -2.0, 0.0], 1),
        ));
        let r = image_directivity(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();
        assert_eq!(values, vec![1.0, -1.0, 0.0]);
    }
}
