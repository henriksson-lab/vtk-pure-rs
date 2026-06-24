//! General 2D convolution with arbitrary kernels.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Convolve image with a 2D kernel.
pub fn convolve_2d(
    input: &ImageData,
    scalars: &str,
    kernel: &[f64],
    kw: usize,
    kh: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let hkw = kw as isize / 2;
    let hkh = kh as isize / 2;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (nx * ny);
            let rem = idx % (nx * ny);
            let iy = rem / nx;
            let ix = rem % nx;
            let mut sum = 0.0;
            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = ix as isize + kx as isize - hkw;
                    let sy = iy as isize + ky as isize - hkh;
                    if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                        sum += vals[sx as usize + sy as usize * nx + iz * nx * ny]
                            * kernel[kx + ky * kw];
                    }
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

/// Sharpen kernel (3x3).
pub fn sharpen_3x3(input: &ImageData, scalars: &str) -> ImageData {
    let k = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    convolve_2d(input, scalars, &k, 3, 3)
}

/// Emboss kernel (3x3).
pub fn emboss_3x3(input: &ImageData, scalars: &str) -> ImageData {
    let k = [-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];
    convolve_2d(input, scalars, &k, 3, 3)
}

/// Box blur with given radius.
pub fn box_blur(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let size = 2 * radius + 1;
    let n = (size * size) as f64;
    let k: Vec<f64> = (0..size * size).map(|_| 1.0 / n).collect();
    convolve_2d(input, scalars, &k, size, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_identity() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let k = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]; // identity
        let r = convolve_2d(&img, "v", &k, 3, 3);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(2 + 2 * 5, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_sharpen() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = sharpen_3x3(&img, "v");
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_box_blur() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = box_blur(&img, "v", 1);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
}
