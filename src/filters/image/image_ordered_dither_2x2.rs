//! 2x2 ordered dither (Bayer)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_ordered_dither_2x2(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let dims = input.dimensions();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let x = i % dims[0];
            let y = (i / dims[0]) % dims[1];
            let threshold = match (x & 1, y & 1) {
                (0, 0) => 0.125,
                (1, 0) => 0.625,
                (0, 1) => 0.875,
                _ => 0.375,
            };
            if buf[0] > threshold {
                1.0
            } else {
                0.0
            }
        })
        .collect();
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
        let r = image_ordered_dither_2x2(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn applies_bayer_thresholds_by_pixel_position() {
        let img = ImageData::with_dimensions(2, 2, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![0.5, 0.5, 0.5, 0.5], 1),
        ));

        let r = image_ordered_dither_2x2(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values, vec![1.0, 0.0, 0.0, 1.0]);
    }
}
