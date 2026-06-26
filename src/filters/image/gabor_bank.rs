//! Gabor filter bank for texture analysis.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a single Gabor filter with given frequency, orientation, and bandwidth.
pub fn gabor_filter(
    input: &ImageData,
    scalars: &str,
    frequency: f64,
    theta: f64,
    sigma: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    if n != nx * ny * dims[2] {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let r = (3.0 * sigma).ceil() as isize;
    let ct = theta.cos();
    let st = theta.sin();
    let s2 = 2.0 * sigma * sigma;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let slice = nx * ny;
            let iz = idx / slice;
            let rem = idx - iz * slice;
            let iy = rem / nx;
            let ix = rem % nx;
            let mut sum = 0.0;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx < 0 || sx >= nx as isize || sy < 0 || sy >= ny as isize {
                        continue;
                    }
                    let xr = dx as f64 * ct + dy as f64 * st;
                    let yr = -dx as f64 * st + dy as f64 * ct;
                    let g = (-((xr * xr + yr * yr) / s2)).exp();
                    let wave = (2.0 * std::f64::consts::PI * frequency * xr).cos();
                    sum += vals[sx as usize + sy as usize * nx] * g * wave;
                }
            }
            sum
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Apply a bank of Gabor filters at multiple orientations and return max response.
pub fn gabor_bank_max(
    input: &ImageData,
    scalars: &str,
    frequency: f64,
    sigma: f64,
    n_orientations: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let n = arr.num_tuples();
    if n != dims[0] * dims[1] * dims[2] {
        return input.clone();
    }
    let mut max_response = vec![0.0f64; n];
    for oi in 0..n_orientations {
        let theta = std::f64::consts::PI * oi as f64 / n_orientations as f64;
        let filtered = gabor_filter(input, scalars, frequency, theta, sigma);
        let farr = filtered.point_data().get_array(scalars).unwrap();
        let mut buf = [0.0f64];
        for i in 0..n {
            farr.tuple_as_f64(i, &mut buf);
            max_response[i] = max_response[i].max(buf[0].abs());
        }
    }
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "GaborMax",
            max_response,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gabor() {
        let img = ImageData::from_function(
            [16, 16, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| (x * 2.0).sin(),
        );
        let r = gabor_filter(&img, "v", 0.3, 0.0, 2.0);
        assert_eq!(r.dimensions(), [16, 16, 1]);
    }
    #[test]
    fn test_bank() {
        let img = ImageData::from_function(
            [12, 12, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| (x * 2.0).sin(),
        );
        let r = gabor_bank_max(&img, "v", 0.3, 2.0, 4);
        assert!(r.point_data().get_array("GaborMax").is_some());
    }

    #[test]
    fn test_gabor_multiple_slices() {
        let img = ImageData::from_function(
            [5, 5, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + y + z * 10.0,
        );
        let r = gabor_filter(&img, "v", 0.3, 0.0, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_tuples(), 50);
    }
}
