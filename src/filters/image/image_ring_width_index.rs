//! Ring width index
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_ring_width_index(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut sum = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        sum += buf[0];
    }
    let mean_width = if n == 0 { 0.0 } else { sum / n as f64 };
    let denominator = mean_width.abs().max(0.01);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] / denominator
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
        let r = image_ring_width_index(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn indexes_against_mean_width() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.0, 4.0],
                1,
            )));
        let r = image_ring_width_index(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0 / 3.0).abs() < 1e-12);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 4.0 / 3.0).abs() < 1e-12);
    }
}
