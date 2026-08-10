//! Lidar pulse width
use crate::data::{AnyDataArray, DataArray, ImageData};

/// Digitizer sampling rate (1 GHz), so one sample lasts 1 ns.
///
/// The width is divided by this rather than multiplied by `1e-9`: `1e-9` is not
/// exactly representable, so `n * 1e-9 * n` rounds twice and drifts off the
/// intended decimal value, while dividing by the exactly representable `1e9`
/// rounds once.
const SAMPLE_RATE_HZ: f64 = 1e9;

/// Smallest sample count used for the width, so a near-zero return still
/// reports a (tiny) non-zero pulse width rather than collapsing to nothing.
const MIN_SAMPLES: f64 = 0.01;

/// Pulse width in seconds for each sample count: `|n| * max(|n|, 0.01)` sampling
/// intervals. The magnitude is taken first, so signed (bipolar) digitizer
/// samples always yield a non-negative width.
pub fn image_pulse_width(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let samples = buf[0].abs();
            samples * samples.max(MIN_SAMPLES) / SAMPLE_RATE_HZ
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
        let r = image_pulse_width(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn pulse_width_is_nonnegative_for_signed_samples() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-2.0, 0.0, 3.0], 1),
        ));

        let r = image_pulse_width(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values, vec![4e-9, 0.0, 9e-9]);
    }
}
