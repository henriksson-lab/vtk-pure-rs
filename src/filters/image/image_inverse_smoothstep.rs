//! Inverse smoothstep
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_inverse_smoothstep(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let data: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.iter()
                .map(|&value| {
                    let y = value.clamp(0.0, 1.0);
                    0.5 - (((1.0 - 2.0 * y).clamp(-1.0, 1.0)).asin() / 3.0).sin()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )));
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
        let r = image_inverse_smoothstep(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn test_inverse_of_cubic_smoothstep() {
        let samples = [0.0, 0.25, 0.5, 0.75, 1.0];
        let mut img = ImageData::with_dimensions(samples.len(), 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                samples.iter().map(|&x| x * x * (3.0 - 2.0 * x)).collect(),
                1,
            )));

        let r = image_inverse_smoothstep(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        for (i, expected) in samples.iter().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn preserves_input_extent() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_extent([5, 7, 10, 12, 2, 2]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.5; 9], 1)));

        let r = image_inverse_smoothstep(&img, "v");
        assert_eq!(r.extent(), [5, 7, 10, 12, 2, 2]);
    }
}
