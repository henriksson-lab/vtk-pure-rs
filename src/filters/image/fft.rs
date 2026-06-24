use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the power spectrum (magnitude squared) of a 1D ImageData signal.
///
/// Uses a direct DFT (not FFT) for simplicity. Suitable for small signals.
/// Adds "PowerSpectrum" array with N/2+1 entries (one-sided spectrum).
/// Only operates on the X dimension (1D signals with ny=nz=1).
pub fn image_power_spectrum(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let n = dims[0] as usize;
    if n < 2 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let signal: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let n_out = n / 2 + 1;
    let mut power = vec![0.0f64; n_out];

    for k in 0..n_out {
        let mut re = 0.0;
        let mut im = 0.0;
        for (j, &x) in signal.iter().enumerate() {
            let angle = -2.0 * std::f64::consts::PI * k as f64 * j as f64 / n as f64;
            re += x * angle.cos();
            im += x * angle.sin();
        }
        power[k] = (re * re + im * im) / (n as f64 * n as f64);
    }

    let mut img = ImageData::with_dimensions(n_out, 1, 1);
    let spacing = input.spacing();
    // Frequency spacing = 1 / (N * dx)
    let freq_spacing = if spacing[0] > 1e-15 {
        1.0 / (n as f64 * spacing[0])
    } else {
        1.0
    };
    img.set_spacing([freq_spacing, 1.0, 1.0]);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PowerSpectrum",
            power,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dc_signal() {
        let mut img = ImageData::with_dimensions(8, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 8], 1)));

        let result = image_power_spectrum(&img, "v");
        let arr = result.point_data().get_array("PowerSpectrum").unwrap();
        let mut buf = [0.0f64];
        // DC component should be 25 (5^2)
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 25.0).abs() < 1e-10);
        // Other frequencies should be ~0
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] < 1e-10);
    }

    #[test]
    fn pure_sine() {
        let n = 16;
        let mut img = ImageData::with_dimensions(n, 1, 1);
        let values: Vec<f64> = (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * 2.0 * i as f64 / n as f64).sin())
            .collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_power_spectrum(&img, "v");
        let arr = result.point_data().get_array("PowerSpectrum").unwrap();
        let mut buf = [0.0f64];
        // Peak at frequency bin 2
        arr.tuple_as_f64(2, &mut buf);
        let peak = buf[0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(peak > buf[0] * 10.0); // much larger than DC
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(4, 1, 1);
        let result = image_power_spectrum(&img, "nope");
        assert_eq!(result.dimensions(), [4, 1, 1]);
    }
}
