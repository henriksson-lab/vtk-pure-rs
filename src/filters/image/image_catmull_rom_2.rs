//! Catmull-Rom basis for interpolation
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_catmull_rom_2(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            {
                let t = buf[0].abs();
                if t <= 1.0 {
                    0.5 * (2.0 + t * t * (-5.0 + 3.0 * t))
                } else if t <= 2.0 {
                    0.5 * (4.0 + t * (-8.0 + t * (5.0 - t)))
                } else {
                    0.0
                }
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
        let r = image_catmull_rom_2(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
