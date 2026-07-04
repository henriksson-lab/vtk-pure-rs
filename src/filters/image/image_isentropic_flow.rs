//! Isentropic flow area ratio
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_isentropic_flow(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            1.0 / buf[0]
                * ((2.0 / (1.4 + 1.0)) * (1.0 + (1.4 - 1.0) / 2.0 * buf[0] * buf[0]))
                    .powf((1.4 + 1.0) / (2.0 * (1.4 - 1.0)))
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
        let r = image_isentropic_flow(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn sonic_mach_has_unit_area_ratio() {
        let img = ImageData::with_dimensions(1, 1, 1)
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0], 1)));
        let r = image_isentropic_flow(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn zero_mach_follows_float_division() {
        let img = ImageData::with_dimensions(1, 1, 1)
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0], 1)));
        let r = image_isentropic_flow(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].is_infinite());
    }

    #[test]
    fn preserves_input_extent() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_extent([5, 7, 10, 12, 2, 2]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let r = image_isentropic_flow(&img, "v");
        assert_eq!(r.extent(), [5, 7, 10, 12, 2, 2]);
    }
}
