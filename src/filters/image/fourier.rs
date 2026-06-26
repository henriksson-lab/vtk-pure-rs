//! Fourier analysis on ImageData: FFT magnitude, phase, filtering.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute 1D DFT magnitude spectrum of each row of a 2D ImageData.
pub fn fft_magnitude_1d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    if dims[2] != 1 || arr.num_tuples() != nx * ny {
        return image.clone();
    }

    let mut buf = [0.0f64];
    let mut magnitude = vec![0.0f64; nx * ny];

    for row in 0..ny {
        // Extract row values
        let row_vals: Vec<f64> = (0..nx)
            .map(|x| {
                let idx = x + row * nx;
                arr.tuple_as_f64(idx, &mut buf);
                buf[0]
            })
            .collect();

        // DFT for this row
        for k in 0..nx {
            let mut re = 0.0;
            let mut im = 0.0;
            for (n, &val) in row_vals.iter().enumerate() {
                let angle = -2.0 * std::f64::consts::PI * k as f64 * n as f64 / nx as f64;
                re += val * angle.cos();
                im += val * angle.sin();
            }
            magnitude[k + row * nx] = (re * re + im * im).sqrt() / nx as f64;
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FFTMagnitude",
            magnitude,
            1,
        )));
    result
}

/// Compute 2D DFT magnitude of a 2D ImageData.
pub fn fft_magnitude_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let n = nx * ny;
    if dims[2] != 1 || arr.num_tuples() != n {
        return image.clone();
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut magnitude = vec![0.0f64; n];

    for ky in 0..ny {
        for kx in 0..nx {
            let mut re = 0.0;
            let mut im = 0.0;
            for y in 0..ny {
                for x in 0..nx {
                    let angle = -2.0
                        * std::f64::consts::PI
                        * (kx as f64 * x as f64 / nx as f64 + ky as f64 * y as f64 / ny as f64);
                    let val = values[x + y * nx];
                    re += val * angle.cos();
                    im += val * angle.sin();
                }
            }
            magnitude[kx + ky * nx] = (re * re + im * im).sqrt() / n as f64;
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FFTMagnitude2D",
            magnitude,
            1,
        )));
    result
}

/// Low-pass filter: zero out frequencies above cutoff.
pub fn low_pass_filter(image: &ImageData, array_name: &str, cutoff_fraction: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let nx = dims[0];
    let ny = dims[1].max(1);
    let n = nx * ny;
    if dims[2] != 1 || arr.num_tuples() != n {
        return image.clone();
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // Forward DFT
    let mut re_f = vec![0.0; n];
    let mut im_f = vec![0.0; n];
    for ky in 0..ny {
        for kx in 0..nx {
            for y in 0..ny {
                for x in 0..nx {
                    let angle = -2.0
                        * std::f64::consts::PI
                        * (kx as f64 * x as f64 / nx as f64 + ky as f64 * y as f64 / ny as f64);
                    let idx = kx + ky * nx;
                    re_f[idx] += values[x + y * nx] * angle.cos();
                    im_f[idx] += values[x + y * nx] * angle.sin();
                }
            }
        }
    }

    // Zero out high frequencies. vtkImageIdealLowPass measures distance from
    // the uncentered frequency origin and wraps coordinates past the midpoint.
    let mid_x = nx as f64 / 2.0;
    let mid_y = ny as f64 / 2.0;
    let cutoff2 = (cutoff_fraction * mid_x.min(mid_y)).powi(2);
    for ky in 0..ny {
        for kx in 0..nx {
            let mut dx = kx as f64;
            let mut dy = ky as f64;
            if dx > mid_x {
                dx = mid_x + mid_x - dx;
            }
            if dy > mid_y {
                dy = mid_y + mid_y - dy;
            }
            if dx * dx + dy * dy > cutoff2 {
                let idx = kx + ky * nx;
                re_f[idx] = 0.0;
                im_f[idx] = 0.0;
            }
        }
    }

    // Inverse DFT
    let mut filtered = vec![0.0f64; n];
    for y in 0..ny {
        for x in 0..nx {
            let mut val = 0.0;
            for ky in 0..ny {
                for kx in 0..nx {
                    let angle = 2.0
                        * std::f64::consts::PI
                        * (kx as f64 * x as f64 / nx as f64 + ky as f64 * y as f64 / ny as f64);
                    let idx = kx + ky * nx;
                    val += re_f[idx] * angle.cos() - im_f[idx] * angle.sin();
                }
            }
            filtered[x + y * nx] = val / n as f64;
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, filtered, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_1d_constant() {
        let img = ImageData::from_function(
            [8, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |_, _, _| 1.0,
        );
        let result = fft_magnitude_1d(&img, "val");
        let arr = result.point_data().get_array("FFTMagnitude").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // DC component
    }

    #[test]
    fn fft_2d() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _| (x * 0.5).sin(),
        );
        let result = fft_magnitude_2d(&img, "val");
        assert!(result.point_data().get_array("FFTMagnitude2D").is_some());
    }

    #[test]
    fn low_pass() {
        let img = ImageData::from_function(
            [16, 16, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, _| (x * 2.0).sin() + (y * 8.0).sin(),
        ); // low + high freq
        let filtered = low_pass_filter(&img, "val", 0.3);
        assert!(filtered.point_data().get_array("val").is_some());
    }
}
