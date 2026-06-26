//! Comoving distance (flat universe)
use crate::data::{AnyDataArray, DataArray, ImageData};

const SPEED_OF_LIGHT_KM_PER_S: f64 = 299_792.458;
const HUBBLE_CONSTANT_KM_PER_S_PER_MPC: f64 = 70.0;

pub fn image_comoving_distance(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            SPEED_OF_LIGHT_KM_PER_S / HUBBLE_CONSTANT_KM_PER_S_PER_MPC * buf[0].max(0.0)
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
        let r = image_comoving_distance(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn zero_redshift_has_zero_distance() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "z",
            |_, _, _| 0.0,
        );
        let r = image_comoving_distance(&img, "z");
        let arr = r.point_data().get_array("z").unwrap();
        let mut buf = [1.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
