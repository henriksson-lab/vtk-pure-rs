//! Simple DFT/inverse DFT for small images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute 2D DFT magnitude spectrum of a small image.
pub fn dft_magnitude(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let pi2 = 2.0 * std::f64::consts::PI;

    let mut mag = vec![0.0f64; n];
    for ky in 0..ny {
        for kx in 0..nx {
            let mut re = 0.0;
            let mut im = 0.0;
            for y in 0..ny {
                for x in 0..nx {
                    let angle =
                        pi2 * (kx as f64 * x as f64 / nx as f64 + ky as f64 * y as f64 / ny as f64);
                    re += vals[x + y * nx] * angle.cos();
                    im -= vals[x + y * nx] * angle.sin();
                }
            }
            mag[kx + ky * nx] = (re * re + im * im).sqrt() / n as f64;
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "DFTMagnitude",
            mag,
            1,
        )))
}

/// Compute power spectrum (log-scaled magnitude^2).
pub fn power_spectrum(input: &ImageData, scalars: &str) -> ImageData {
    let dft = dft_magnitude(input, scalars);
    let arr = dft.point_data().get_array("DFTMagnitude").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (1.0 + buf[0] * buf[0]).ln()
        })
        .collect();
    let dims = dft.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(dft.spacing())
        .with_origin(dft.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "PowerSpectrum",
            data,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dft() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| (x * std::f64::consts::PI).sin(),
        );
        let r = dft_magnitude(&img, "v");
        assert_eq!(r.dimensions(), [8, 8, 1]);
        assert!(r.point_data().get_array("DFTMagnitude").is_some());
    }
    #[test]
    fn test_power() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = power_spectrum(&img, "v");
        assert!(r.point_data().get_array("PowerSpectrum").is_some());
    }
}
