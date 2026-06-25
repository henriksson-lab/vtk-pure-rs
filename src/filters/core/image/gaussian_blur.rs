use crate::data::{AnyDataArray, DataArray, ImageData};

fn compute_kernel(min: isize, max: isize, sigma: f64) -> Vec<f64> {
    if sigma == 0.0 {
        return vec![1.0];
    }

    let mut kernel = Vec::with_capacity((max - min + 1) as usize);
    let mut sum = 0.0;
    for x in min..=max {
        let weight = (-(x * x) as f64 / (sigma * sigma * 2.0)).exp();
        kernel.push(weight);
        sum += weight;
    }
    for weight in &mut kernel {
        *weight /= sum;
    }
    kernel
}

fn convolve_axis(
    input: &[f64],
    output: &mut [f64],
    dims: [usize; 3],
    axis: usize,
    radius: usize,
    sigma: f64,
) {
    if sigma == 0.0 || radius == 0 {
        output.copy_from_slice(input);
        return;
    }

    let [nx, ny, nz] = dims;
    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };
    let axis_len = dims[axis];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let coord = [i, j, k][axis];
                let left_clip = radius.saturating_sub(coord);
                let right_clip = (coord + radius).saturating_sub(axis_len - 1);
                let min = -(radius as isize) + left_clip as isize;
                let max = radius as isize - right_clip as isize;
                let kernel = compute_kernel(min, max, sigma);

                let mut acc = 0.0;
                for (offset, weight) in (min..=max).zip(kernel.iter()) {
                    let mut pos = [i, j, k];
                    pos[axis] = (coord as isize + offset) as usize;
                    acc += input[idx(pos[0], pos[1], pos[2])] * weight;
                }
                output[idx(i, j, k)] = acc;
            }
        }
    }
}

/// Apply a Gaussian blur to a named scalar array on an ImageData.
///
/// Uses a separable 1D Gaussian kernel along each axis. The kernel radius
/// is derived from `sigma` as `ceil(3 * sigma)`. Adds a "Blurred" array
/// to the output point data.
pub fn gaussian_blur(input: &ImageData, scalars: &str, sigma: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0];
    let ny: usize = dims[1];
    let nz: usize = dims[2];
    let n: usize = nx * ny * nz;

    // Read source values
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let sigma: f64 = sigma.max(0.0);
    let r: usize = (3.0 * sigma).ceil() as usize;

    let mut tmp: Vec<f64> = vec![0.0; n];
    let mut tmp2: Vec<f64> = vec![0.0; n];
    let mut result: Vec<f64> = vec![0.0; n];
    let dims = [nx, ny, nz];
    convolve_axis(&values, &mut tmp, dims, 2, r, sigma);
    convolve_axis(&tmp, &mut tmp2, dims, 1, r, sigma);
    convolve_axis(&tmp2, &mut result, dims, 0, r, sigma);

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Blurred", result, 1)));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSet;

    #[test]
    fn blur_preserves_dimensions() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = img.num_points();
        let scalars: Vec<f64> = vec![1.0; n];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", scalars, 1)));

        let out = gaussian_blur(&img, "S", 1.0);
        assert_eq!(out.dimensions(), [5, 5, 5]);
        let blurred = out.point_data().get_array("Blurred").unwrap();
        assert_eq!(blurred.num_tuples(), n);
    }

    #[test]
    fn uniform_field_stays_uniform() {
        let mut img = ImageData::with_dimensions(7, 7, 7);
        let n: usize = img.num_points();
        let val: f64 = 3.5;
        let scalars: Vec<f64> = vec![val; n];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", scalars, 1)));

        let out = gaussian_blur(&img, "S", 1.5);
        let blurred = out.point_data().get_array("Blurred").unwrap();
        let mut buf = [0.0f64];
        // Check center voxel — should still be ~3.5
        let center: usize = n / 2;
        blurred.tuple_as_f64(center, &mut buf);
        assert!((buf[0] - val).abs() < 1e-10);
    }

    #[test]
    fn blur_reduces_peak() {
        let mut img = ImageData::with_dimensions(11, 11, 11);
        let n: usize = img.num_points();
        let mut scalars: Vec<f64> = vec![0.0; n];
        // Single hot voxel in the center
        let cx: usize = 5;
        let cy: usize = 5;
        let cz: usize = 5;
        let center_idx: usize = cz * 11 * 11 + cy * 11 + cx;
        scalars[center_idx] = 100.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", scalars, 1)));

        let out = gaussian_blur(&img, "S", 2.0);
        let blurred = out.point_data().get_array("Blurred").unwrap();
        let mut buf = [0.0f64];
        blurred.tuple_as_f64(center_idx, &mut buf);
        // Peak should be reduced significantly
        assert!(buf[0] < 50.0);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn zero_sigma_is_identity() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        let scalars = vec![1.0, 5.0, 9.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", scalars, 1)));

        let out = gaussian_blur(&img, "S", 0.0);
        let blurred = out.point_data().get_array("Blurred").unwrap();
        let mut buf = [0.0f64];
        for (idx, expected) in [1.0, 5.0, 9.0].into_iter().enumerate() {
            blurred.tuple_as_f64(idx, &mut buf);
            assert_eq!(buf[0], expected);
        }
    }
}
