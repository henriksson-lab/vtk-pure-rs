//! Precipitation pH threshold
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_precipitation_ph(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let hydroxide = buf[0].max(0.0);
            let ratio = (hydroxide / 0.01).clamp(1e-14, 1.0);
            14.0 + ratio.log10()
        })
        .collect();
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output.set_extent(input.extent());
    output
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
        let r = image_precipitation_ph(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn thresholds_precipitation_ph_without_nan() {
        let img = ImageData::with_dimensions(5, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-1.0, 0.0, 0.001, 0.01, 0.1], 1),
        ));

        let r = image_precipitation_ph(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values, vec![0.0, 0.0, 13.0, 14.0, 14.0]);
        assert!(values.iter().all(|v| v.is_finite()));
    }
}
