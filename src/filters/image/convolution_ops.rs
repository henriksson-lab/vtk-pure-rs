//! Image convolution operations: Sobel, Prewitt, Roberts, Scharr.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply Sobel edge detection to a 2D image.
pub fn sobel_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    let v = |x: usize, y: usize| -> f64 {
        if x < dims[0] && y < dims[1] {
            vals[x + y * dims[0]]
        } else {
            0.0
        }
    };

    let mut mag = vec![0.0f64; n];
    for y in 1..dims[1].saturating_sub(1) {
        for x in 1..dims[0].saturating_sub(1) {
            let gx = -v(x - 1, y - 1) - 2.0 * v(x - 1, y) - v(x - 1, y + 1)
                + v(x + 1, y - 1)
                + 2.0 * v(x + 1, y)
                + v(x + 1, y + 1);
            let gy = -v(x - 1, y - 1) - 2.0 * v(x, y - 1) - v(x + 1, y - 1)
                + v(x - 1, y + 1)
                + 2.0 * v(x, y + 1)
                + v(x + 1, y + 1);
            mag[x + y * dims[0]] = (gx * gx + gy * gy).sqrt();
        }
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SobelMagnitude",
            mag,
            1,
        )));
    r
}

/// Apply Prewitt edge detection.
pub fn prewitt_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    let v = |x: usize, y: usize| -> f64 {
        if x < dims[0] && y < dims[1] {
            vals[x + y * dims[0]]
        } else {
            0.0
        }
    };

    let mut mag = vec![0.0f64; n];
    for y in 1..dims[1].saturating_sub(1) {
        for x in 1..dims[0].saturating_sub(1) {
            let gx = -v(x - 1, y - 1) - v(x - 1, y) - v(x - 1, y + 1)
                + v(x + 1, y - 1)
                + v(x + 1, y)
                + v(x + 1, y + 1);
            let gy = -v(x - 1, y - 1) - v(x, y - 1) - v(x + 1, y - 1)
                + v(x - 1, y + 1)
                + v(x, y + 1)
                + v(x + 1, y + 1);
            mag[x + y * dims[0]] = (gx * gx + gy * gy).sqrt();
        }
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PrewittMagnitude",
            mag,
            1,
        )));
    r
}

/// Apply Scharr edge detection (more accurate than Sobel).
pub fn scharr_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    let v = |x: usize, y: usize| -> f64 {
        if x < dims[0] && y < dims[1] {
            vals[x + y * dims[0]]
        } else {
            0.0
        }
    };

    let mut mag = vec![0.0f64; n];
    for y in 1..dims[1].saturating_sub(1) {
        for x in 1..dims[0].saturating_sub(1) {
            let gx = -3.0 * v(x - 1, y - 1) - 10.0 * v(x - 1, y) - 3.0 * v(x - 1, y + 1)
                + 3.0 * v(x + 1, y - 1)
                + 10.0 * v(x + 1, y)
                + 3.0 * v(x + 1, y + 1);
            let gy = -3.0 * v(x - 1, y - 1) - 10.0 * v(x, y - 1) - 3.0 * v(x + 1, y - 1)
                + 3.0 * v(x - 1, y + 1)
                + 10.0 * v(x, y + 1)
                + 3.0 * v(x + 1, y + 1);
            mag[x + y * dims[0]] = (gx * gx + gy * gy).sqrt();
        }
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScharrMagnitude",
            mag,
            1,
        )));
    r
}

/// Apply Gaussian blur with given sigma.
pub fn gaussian_blur_2d(image: &ImageData, array_name: &str, sigma: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let radius = (sigma * 3.0).ceil() as i64;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();

    let mut output = vec![0.0f64; n];
    for y in 0..dims[1] {
        for x in 0..dims[0] {
            let mut sum = 0.0;
            let mut w_sum = 0.0;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i64 + dx;
                    let ny = y as i64 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < dims[0] && (ny as usize) < dims[1] {
                        let w = (-(dx * dx + dy * dy) as f64 / (2.0 * sigma * sigma)).exp();
                        sum += w * vals[nx as usize + ny as usize * dims[0]];
                        w_sum += w;
                    }
                }
            }
            output[x + y * dims[0]] = if w_sum > 1e-15 { sum / w_sum } else { 0.0 };
        }
    }
    let mut r = image.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sobel() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| if x > 10.0 { 1.0 } else { 0.0 },
        );
        let r = sobel_2d(&img, "v");
        assert!(r.point_data().get_array("SobelMagnitude").is_some());
    }
    #[test]
    fn prewitt() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = prewitt_2d(&img, "v");
        assert!(r.point_data().get_array("PrewittMagnitude").is_some());
    }
    #[test]
    fn gaussian() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| if x == 5.0 && y == 5.0 { 1.0 } else { 0.0 },
        );
        let r = gaussian_blur_2d(&img, "v", 1.0);
        assert!(r.point_data().get_array("v").is_some());
    }
}
