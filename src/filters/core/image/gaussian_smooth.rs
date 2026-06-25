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

/// Apply Gaussian smoothing to an ImageData scalar field.
///
/// Performs a 3D Gaussian blur with the given `sigma` (in voxel units)
/// and `radius` (kernel half-size in voxels). Uses separable 1D passes
/// along each axis for efficiency.
pub fn image_gaussian_smooth(
    input: &ImageData,
    scalars: &str,
    sigma: f64,
    radius: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let sigma = sigma.max(0.0);
    let r = radius;

    let mut tmp = vec![0.0f64; n];
    let mut tmp2 = vec![0.0f64; n];
    let mut result = vec![0.0f64; n];
    let dims = [nx, ny, nz];
    convolve_axis(&values, &mut tmp, dims, 2, r, sigma);
    convolve_axis(&tmp, &mut tmp2, dims, 1, r, sigma);
    convolve_axis(&tmp2, &mut result, dims, 0, r, sigma);

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spike_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n = 125;
        let mut values = vec![0.0f64; n];
        // Spike at center
        values[62] = 100.0; // (2,2,2)
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn smoothing_reduces_spike() {
        let img = make_spike_image();
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf);
        // Spike should be reduced
        assert!(buf[0] < 100.0, "center={}", buf[0]);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn smoothing_spreads() {
        let img = make_spike_image();
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        // Neighbor should pick up some value
        arr.tuple_as_f64(63, &mut buf); // (3,2,2)
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn preserves_uniform() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![5.0; 27],
                1,
            )));
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn zero_sigma_is_identity() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 5.0, 9.0],
                1,
            )));
        let result = image_gaussian_smooth(&img, "val", 0.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        for (idx, expected) in [1.0, 5.0, 9.0].into_iter().enumerate() {
            arr.tuple_as_f64(idx, &mut buf);
            assert_eq!(buf[0], expected);
        }
    }
}
