//! Diffraction grating order
use crate::data::{AnyDataArray, DataArray, ImageData};

/// Number of illuminated slits in the modelled grating.
const SLITS: f64 = 3.0;

/// Relative intensity of a `SLITS`-slit grating.
///
/// The scalar is the normalized diffraction variable `u`; the grating phase is
/// `x = 2*pi*u` and the response is the single-slit envelope times the `N`-slit
/// interference factor:
///
/// ```text
/// I(u) = sinc(x)^2 * [sin(N*x) / (N*sin(x))]^2,   x = 2*pi*u
/// ```
///
/// The slit count enters the interference factor only, never the phase, which
/// puts the `N - 1` interference minima evenly between successive principal
/// maxima -- for three slits, `u = 1/6` and `u = 1/3` between the orders at
/// `u = 0` and `u = 1/2`.
pub fn image_diffraction_grating(input: &ImageData, scalars: &str) -> ImageData {
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
                let x = buf[0] * std::f64::consts::TAU;
                if x.abs() < 1e-15 {
                    1.0
                } else {
                    let sinc = x.sin() / x;
                    let denominator = SLITS * x.sin();
                    let grating = if denominator.abs() < 1e-15 {
                        1.0
                    } else {
                        (SLITS * x).sin() / denominator
                    };
                    sinc.powi(2) * grating.powi(2)
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

    fn image(values: &[f64]) -> ImageData {
        ImageData::with_dimensions(values.len(), 1, 1)
            .with_spacing([0.5, 2.0, 1.0])
            .with_origin([1.0, -1.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                values.to_vec(),
                1,
            )))
    }

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= 1e-12, "{a} != {e}");
        }
    }

    #[test]
    fn computes_three_slit_grating_response() {
        let img = image(&[0.0, 1.0 / 6.0, 1.0 / 3.0]);
        let r = image_diffraction_grating(&img, "v");
        assert_eq!(r.dimensions(), [3, 1, 1]);
        assert_eq!(r.spacing(), img.spacing());
        assert_eq!(r.origin(), img.origin());
        assert_close(
            &r.point_data().get_array("v").unwrap().to_f64_vec(),
            &[1.0, 0.0, 0.0],
        );
    }
}
