//! Bloch oscillation in lattice
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_bloch_oscillation(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let phase = buf[0] * std::f64::consts::PI;
            if phase.abs() < 1e-15 {
                1.0
            } else {
                phase.sin() / phase
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
        let r = image_bloch_oscillation(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn preserves_sinc_symmetry_and_origin() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-0.5, 0.0, 0.5], 1),
        ));
        let r = image_bloch_oscillation(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut value = [0.0f64];

        arr.tuple_as_f64(0, &mut value);
        let negative = value[0];
        arr.tuple_as_f64(1, &mut value);
        assert_eq!(value[0], 1.0);
        arr.tuple_as_f64(2, &mut value);
        assert!((negative - value[0]).abs() < 1e-15);
    }
}
