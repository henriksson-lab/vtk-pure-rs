//! Rescale to [0,1] by min-max
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_rescale_minmax(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut data: Vec<f64> = Vec::with_capacity(n);
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_value = min_value.min(buf[0]);
        max_value = max_value.max(buf[0]);
        data.push(buf[0]);
    }
    let range = max_value - min_value;
    if range > 1e-30 {
        for value in &mut data {
            *value = (*value - min_value) / range;
        }
    } else {
        data.fill(0.0);
    }
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
        let r = image_rescale_minmax(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn rescales_to_unit_interval() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.0, 4.0, 6.0],
                1,
            )));

        let r = image_rescale_minmax(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-12);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-12);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn constant_input_maps_to_zero() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 3.0],
                1,
            )));

        let r = image_rescale_minmax(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [1.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
