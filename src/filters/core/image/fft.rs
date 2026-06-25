use crate::data::{AnyDataArray, DataArray, ImageData};

#[derive(Clone, Copy, Debug, Default)]
struct ImageComplex {
    real: f64,
    imag: f64,
}

impl ImageComplex {
    fn add(self, other: Self) -> Self {
        Self {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
        }
    }

    fn exponential(self) -> Self {
        let tmp = self.real.exp();
        Self {
            real: tmp * self.imag.cos(),
            imag: tmp * self.imag.sin(),
        }
    }
}

/// Perform VTK-style `vtkImageFFT` over the image axes.
///
/// The named point-data array supplies the real input in component 0 and, when
/// present, the imaginary input in component 1. The output is a double array
/// with two components: real in component 0 and imaginary in component 1.
pub fn image_fft(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() >= 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let number_of_points = dims[0].saturating_mul(dims[1]).saturating_mul(dims[2]);
    if number_of_points == 0 || arr.num_tuples() < number_of_points {
        return input.clone();
    }

    let mut tuple = [0.0f64; 2];
    let mut data: Vec<ImageComplex> = (0..number_of_points)
        .map(|idx| {
            tuple = [0.0, 0.0];
            arr.tuple_as_f64(idx, &mut tuple);
            ImageComplex {
                real: tuple[0],
                imag: tuple[1],
            }
        })
        .collect();

    for iteration in 0..3 {
        execute_axis_fft(&mut data, dims, iteration);
    }

    let mut output_values = Vec::with_capacity(number_of_points * 2);
    for value in data {
        output_values.push(value.real);
        output_values.push(value.imag);
    }

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            output_values,
            2,
        )));
    output.point_data_mut().set_active_scalars(scalars);
    output
}

/// Backwards-compatible entry point for the old helper name.
///
/// This file is the `vtkImageFFT` translation, so the function now returns the
/// same two-component complex image data as [`image_fft`].
pub fn image_power_spectrum(input: &ImageData, scalars: &str) -> ImageData {
    image_fft(input, scalars)
}

fn execute_axis_fft(data: &mut [ImageComplex], dims: [usize; 3], iteration: usize) {
    let in_size0 = dims[iteration];
    if in_size0 == 0 {
        return;
    }

    let (other_axis1, other_axis2) = match iteration {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let mut in_complex = vec![ImageComplex::default(); in_size0];
    let mut out_complex = vec![ImageComplex::default(); in_size0];

    for idx2 in 0..dims[other_axis2] {
        for idx1 in 0..dims[other_axis1] {
            for idx0 in 0..in_size0 {
                let mut ijk = [0usize; 3];
                ijk[iteration] = idx0;
                ijk[other_axis1] = idx1;
                ijk[other_axis2] = idx2;
                in_complex[idx0] = data[flat_index(ijk[0], ijk[1], ijk[2], dims)];
            }

            execute_fft(&mut in_complex, &mut out_complex, in_size0);

            for (idx0, value) in out_complex.iter().enumerate() {
                let mut ijk = [0usize; 3];
                ijk[iteration] = idx0;
                ijk[other_axis1] = idx1;
                ijk[other_axis2] = idx2;
                data[flat_index(ijk[0], ijk[1], ijk[2], dims)] = *value;
            }
        }
    }
}

fn flat_index(i: usize, j: usize, k: usize, dims: [usize; 3]) -> usize {
    i + j * dims[0] + k * dims[0] * dims[1]
}

fn execute_fft_step2(
    p_in: &[ImageComplex],
    p_out: &mut [ImageComplex],
    n: usize,
    bsize: usize,
    fb: i32,
) {
    let mut p1 = 0usize;
    let mut p3 = 0usize;
    for _i1 in 0..n / (bsize * 2) {
        let mut p2 = p1;
        for _i2 in 0..bsize {
            p_out[p3] = p_in[p2];
            p2 += 1;
            p3 += 1;
        }
        p2 = p1;
        for _i2 in 0..bsize {
            p_out[p3] = p_in[p2];
            p2 += 1;
            p3 += 1;
        }
        p1 += bsize;
    }

    let fact1 = ImageComplex {
        real: 1.0,
        imag: 0.0,
    };
    let q = ImageComplex {
        real: 0.0,
        imag: -(2.0 * std::f64::consts::PI) * fb as f64 / (bsize as f64 * 2.0),
    }
    .exponential();
    p3 = 0;
    for _i1 in 0..n / (bsize * 2) {
        let mut fact = fact1;
        let mut p2 = p1;
        for _i2 in 0..bsize {
            let temp = fact.multiply(p_in[p2]);
            p_out[p3] = temp.add(p_out[p3]);
            fact = q.multiply(fact);
            p2 += 1;
            p3 += 1;
        }
        p2 = p1;
        for _i2 in 0..bsize {
            let temp = fact.multiply(p_in[p2]);
            p_out[p3] = temp.add(p_out[p3]);
            fact = q.multiply(fact);
            p2 += 1;
            p3 += 1;
        }
        p1 += bsize;
    }
}

fn execute_fft_step_n(
    p_in: &[ImageComplex],
    p_out: &mut [ImageComplex],
    n_total: usize,
    bsize: usize,
    n: usize,
    fb: i32,
) {
    for value in p_out.iter_mut().take(n_total) {
        *value = ImageComplex::default();
    }

    let mut p1 = 0usize;
    for i0 in 0..n {
        let q = ImageComplex {
            real: 0.0,
            imag: -(2.0 * std::f64::consts::PI) * i0 as f64 * fb as f64 / (bsize as f64 * n as f64),
        }
        .exponential();
        let mut p3 = 0usize;
        for _i1 in 0..n_total / (bsize * n) {
            let mut fact = ImageComplex {
                real: 1.0,
                imag: 0.0,
            };
            for _i3 in 0..n {
                let mut p2 = p1;
                for _i2 in 0..bsize {
                    let temp = fact.multiply(p_in[p2]);
                    p_out[p3] = temp.add(p_out[p3]);
                    fact = q.multiply(fact);
                    p2 += 1;
                    p3 += 1;
                }
            }
            p1 += bsize;
        }
    }
}

fn execute_fft_forward_backward(
    in_complex: &mut [ImageComplex],
    out_complex: &mut [ImageComplex],
    n_total: usize,
    fb: i32,
) {
    let mut block_size = 1usize;
    let mut rest_size = n_total;
    let mut n = 2usize;

    if fb == -1 {
        for value in in_complex.iter_mut().take(n_total) {
            value.real /= n_total as f64;
            value.imag /= n_total as f64;
        }
    }

    let mut first = in_complex.to_vec();
    let mut second = out_complex.to_vec();
    let mut first_is_current = true;

    while block_size < n_total && n <= n_total {
        if rest_size % n == 0 {
            if first_is_current {
                if n == 2 {
                    execute_fft_step2(&first, &mut second, n_total, block_size, fb);
                } else {
                    execute_fft_step_n(&first, &mut second, n_total, block_size, n, fb);
                }
            } else if n == 2 {
                execute_fft_step2(&second, &mut first, n_total, block_size, fb);
            } else {
                execute_fft_step_n(&second, &mut first, n_total, block_size, n, fb);
            }
            block_size *= n;
            rest_size /= n;
            first_is_current = !first_is_current;
        } else {
            n += 1;
        }
    }

    let current = if first_is_current { &first } else { &second };
    out_complex[..n_total].copy_from_slice(&current[..n_total]);
}

fn execute_fft(in_complex: &mut [ImageComplex], out_complex: &mut [ImageComplex], n: usize) {
    execute_fft_forward_backward(in_complex, out_complex, n, 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuple(result: &ImageData, name: &str, idx: usize) -> [f64; 2] {
        let arr = result.point_data().get_array(name).unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(idx, &mut buf);
        buf
    }

    #[test]
    fn constant_signal_is_complex_dc_output() {
        let mut img = ImageData::with_dimensions(8, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 8], 1)));

        let result = image_fft(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        assert_eq!(result.dimensions(), [8, 1, 1]);

        let dc = tuple(&result, "v", 0);
        assert!((dc[0] - 40.0).abs() < 1e-10);
        assert!(dc[1].abs() < 1e-10);
        for idx in 1..8 {
            let value = tuple(&result, "v", idx);
            assert!(value[0].abs() < 1e-10);
            assert!(value[1].abs() < 1e-10);
        }
    }

    #[test]
    fn complex_input_uses_first_two_components() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                2,
            )));

        let result = image_fft(&img, "v");
        let dc = tuple(&result, "v", 0);
        assert!((dc[0] - 4.0).abs() < 1e-10);
        assert!((dc[1] - 8.0).abs() < 1e-10);
    }

    #[test]
    fn non_power_of_two_matches_forward_dft() {
        let values = vec![1.0, 2.0, 4.0];
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                values.clone(),
                1,
            )));

        let result = image_fft(&img, "v");
        for k in 0..3 {
            let mut re = 0.0;
            let mut im = 0.0;
            for (j, x) in values.iter().enumerate() {
                let angle = -2.0 * std::f64::consts::PI * k as f64 * j as f64 / 3.0;
                re += x * angle.cos();
                im += x * angle.sin();
            }
            let value = tuple(&result, "v", k);
            assert!((value[0] - re).abs() < 1e-10);
            assert!((value[1] - im).abs() < 1e-10);
        }
    }

    #[test]
    fn transforms_over_image_axes() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.0, 0.0],
                1,
            )));

        let result = image_fft(&img, "v");
        for idx in 0..4 {
            let value = tuple(&result, "v", idx);
            assert!((value[0] - 1.0).abs() < 1e-10);
            assert!(value[1].abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(4, 1, 1);
        let result = image_fft(&img, "nope");
        assert_eq!(result.dimensions(), [4, 1, 1]);
        assert_eq!(result.point_data().num_arrays(), 0);
    }
}
