//! Microfluidic Peclet number
use crate::data::{AnyDataArray, DataArray, ImageData};

/// Ratio of the assumed microchannel length (1 um) to the assumed solute
/// diffusivity (1e-9 m^2/s), i.e. `L / D = 1e-6 / 1e-9 = 1e3` s/m.
///
/// Stored pre-divided: evaluating `v * 1e-6 / 1e-9` rounds twice (and neither
/// `1e-6` nor `1e-9` is exactly representable), which loses the last digits of
/// the result. The exact decimal ratio only rounds once.
const LENGTH_OVER_DIFFUSIVITY: f64 = 1e3;

/// Peclet number `Pe = v * L / D` for each sample, where `v` is the scalar
/// (advective velocity in m/s), `L = 1` um and `D = 1e-9` m^2/s.
pub fn image_peclet_microfluidic(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] * LENGTH_OVER_DIFFUSIVITY
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
        let r = image_peclet_microfluidic(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn uses_microchannel_length_and_diffusivity() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![0.0, 0.5, 2.0], 1),
        ));

        let r = image_peclet_microfluidic(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values, vec![0.0, 500.0, 2000.0]);
    }
}
