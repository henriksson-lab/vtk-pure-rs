//! ImageFFT — true 2D FFT on ImageData using row/column Cooley-Tukey.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Perform a 2D FFT on a 2D ImageData scalar field.
///
/// Input: 2D ImageData with a scalar array named `scalar_name`.
/// Output: ImageData with "Magnitude" and "Phase" arrays.
///
/// Uses Cooley-Tukey radix-2 FFT. Dimensions are zero-padded to the next
/// power of two.
pub fn image_fft_real(input: &ImageData, scalar_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let n = arr.num_tuples();

    // Read input data
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // Pad to next power of two
    let fft_nx = nx.next_power_of_two();
    let fft_ny = ny.next_power_of_two();

    // Create complex array (real, imag interleaved)
    let mut re = vec![0.0f64; fft_nx * fft_ny];
    let mut im = vec![0.0f64; fft_nx * fft_ny];

    // Copy input into padded array
    for iy in 0..ny {
        for ix in 0..nx {
            re[iy * fft_nx + ix] = vals[iy * nx + ix];
        }
    }

    // Row-wise FFT
    for iy in 0..fft_ny {
        let start = iy * fft_nx;
        let end = start + fft_nx;
        fft_in_place(&mut re[start..end], &mut im[start..end]);
    }

    // Column-wise FFT: extract columns, FFT, put back
    let mut col_re = vec![0.0f64; fft_ny];
    let mut col_im = vec![0.0f64; fft_ny];
    for ix in 0..fft_nx {
        for iy in 0..fft_ny {
            col_re[iy] = re[iy * fft_nx + ix];
            col_im[iy] = im[iy * fft_nx + ix];
        }
        fft_in_place(&mut col_re, &mut col_im);
        for iy in 0..fft_ny {
            re[iy * fft_nx + ix] = col_re[iy];
            im[iy * fft_nx + ix] = col_im[iy];
        }
    }

    // Compute magnitude and phase for original dimensions
    let mut magnitude = vec![0.0f64; nx * ny];
    let mut phase = vec![0.0f64; nx * ny];
    for iy in 0..ny {
        for ix in 0..nx {
            let r = re[iy * fft_nx + ix];
            let i = im[iy * fft_nx + ix];
            magnitude[iy * nx + ix] = (r * r + i * i).sqrt();
            phase[iy * nx + ix] = i.atan2(r);
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Magnitude",
            magnitude,
            1,
        )))
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Phase", phase, 1)))
}

/// In-place Cooley-Tukey radix-2 FFT.
fn fft_in_place(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    if n <= 1 {
        return;
    }
    assert!(n.is_power_of_two(), "FFT length must be power of two");

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterfly operations
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * std::f64::consts::PI / len as f64;
        let wn_re = angle.cos();
        let wn_im = angle.sin();

        let mut i = 0;
        while i < n {
            let mut w_re = 1.0;
            let mut w_im = 0.0;
            for k in 0..half {
                let u_re = re[i + k];
                let u_im = im[i + k];
                let t_re = w_re * re[i + k + half] - w_im * im[i + k + half];
                let t_im = w_re * im[i + k + half] + w_im * re[i + k + half];
                re[i + k] = u_re + t_re;
                im[i + k] = u_im + t_im;
                re[i + k + half] = u_re - t_re;
                im[i + k + half] = u_im - t_im;
                let new_w_re = w_re * wn_re - w_im * wn_im;
                let new_w_im = w_re * wn_im + w_im * wn_re;
                w_re = new_w_re;
                w_im = new_w_im;
            }
            i += len;
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_constant_signal() {
        // Constant signal should have all energy in DC component
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |_, _, _| 1.0,
        );
        let result = image_fft_real(&img, "val");
        let mag = result.point_data().get_array("Magnitude").unwrap();
        let phase = result.point_data().get_array("Phase").unwrap();
        assert_eq!(mag.num_tuples(), 16);
        assert_eq!(phase.num_tuples(), 16);

        // DC component (index 0) should have magnitude = N (16)
        let mut buf = [0.0f64];
        mag.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 16.0).abs() < 1e-6, "DC magnitude = {}", buf[0]);
    }

    #[test]
    fn fft_produces_both_arrays() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "s",
            |x, y, _| (x + y) as f64,
        );
        let result = image_fft_real(&img, "s");
        assert!(result.point_data().get_array("Magnitude").is_some());
        assert!(result.point_data().get_array("Phase").is_some());
    }

    #[test]
    fn fft_missing_array() {
        let img = ImageData::with_dimensions(4, 4, 1);
        let result = image_fft_real(&img, "nonexistent");
        assert_eq!(result.dimensions(), [4, 4, 1]);
    }
}
