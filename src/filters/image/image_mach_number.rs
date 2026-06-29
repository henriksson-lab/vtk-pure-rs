//! Mach number
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_mach_number(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0].abs()
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
        let r = image_mach_number(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn maps_scalar_to_mach_magnitude() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-2.0, 0.5, 3.0], 1),
        ));

        let r = image_mach_number(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0];

        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple[0], 2.0);
        arr.tuple_as_f64(1, &mut tuple);
        assert_eq!(tuple[0], 0.5);
        arr.tuple_as_f64(2, &mut tuple);
        assert_eq!(tuple[0], 3.0);
    }
}
