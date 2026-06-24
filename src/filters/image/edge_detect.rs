//! Edge detection filters (Sobel, Prewitt, Roberts, Laplacian).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Sobel edge detection (gradient magnitude).
pub fn sobel_edge(input: &ImageData, scalars: &str) -> ImageData {
    let (gx, gy) = sobel_gradients(input, scalars);
    let n = gx.len();
    let data: Vec<f64> = (0..n)
        .map(|i| (gx[i] * gx[i] + gy[i] * gy[i]).sqrt())
        .collect();
    make_result(input, scalars, data)
}

/// Prewitt edge detection.
pub fn prewitt_edge(input: &ImageData, scalars: &str) -> ImageData {
    let vals = read_vals(input, scalars);
    if vals.is_empty() {
        return input.clone();
    }
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = vals.len();
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = (idx / nx) % ny;
            let ix = idx % nx;
            if ix == 0 || ix >= nx - 1 || iy == 0 || iy >= ny - 1 {
                return 0.0;
            }
            let g = |dx: isize, dy: isize| {
                vals[(ix as isize + dx) as usize + (iy as isize + dy) as usize * nx]
            };
            let gx = g(1, -1) + g(1, 0) + g(1, 1) - g(-1, -1) - g(-1, 0) - g(-1, 1);
            let gy = g(-1, 1) + g(0, 1) + g(1, 1) - g(-1, -1) - g(0, -1) - g(1, -1);
            (gx * gx + gy * gy).sqrt()
        })
        .collect();
    make_result(input, scalars, data)
}

/// Roberts cross edge detection.
pub fn roberts_edge(input: &ImageData, scalars: &str) -> ImageData {
    let vals = read_vals(input, scalars);
    if vals.is_empty() {
        return input.clone();
    }
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = vals.len();
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = (idx / nx) % ny;
            let ix = idx % nx;
            if ix >= nx - 1 || iy >= ny - 1 {
                return 0.0;
            }
            let g = |dx: usize, dy: usize| vals[(ix + dx) + (iy + dy) * nx];
            let gx = g(1, 0) - g(0, 1);
            let gy = g(1, 1) - g(0, 0);
            (gx * gx + gy * gy).sqrt()
        })
        .collect();
    make_result(input, scalars, data)
}

/// Laplacian edge detection (second derivative).
pub fn laplacian_edge(input: &ImageData, scalars: &str) -> ImageData {
    let vals = read_vals(input, scalars);
    if vals.is_empty() {
        return input.clone();
    }
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = vals.len();
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = (idx / nx) % ny;
            let ix = idx % nx;
            if ix == 0 || ix >= nx - 1 || iy == 0 || iy >= ny - 1 {
                return 0.0;
            }
            let c = vals[idx];
            let lap = vals[idx - 1] + vals[idx + 1] + vals[idx - nx] + vals[idx + nx] - 4.0 * c;
            lap.abs()
        })
        .collect();
    make_result(input, scalars, data)
}

fn sobel_gradients(input: &ImageData, scalars: &str) -> (Vec<f64>, Vec<f64>) {
    let vals = read_vals(input, scalars);
    if vals.is_empty() {
        return (vec![], vec![]);
    }
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = vals.len();
    let mut gx = vec![0.0; n];
    let mut gy = vec![0.0; n];
    for idx in 0..n {
        let iy = (idx / nx) % ny;
        let ix = idx % nx;
        if ix == 0 || ix >= nx - 1 || iy == 0 || iy >= ny - 1 {
            continue;
        }
        let g = |dx: isize, dy: isize| {
            vals[(ix as isize + dx) as usize + (iy as isize + dy) as usize * nx]
        };
        gx[idx] = g(1, -1) + 2.0 * g(1, 0) + g(1, 1) - g(-1, -1) - 2.0 * g(-1, 0) - g(-1, 1);
        gy[idx] = g(-1, 1) + 2.0 * g(0, 1) + g(1, 1) - g(-1, -1) - 2.0 * g(0, -1) - g(1, -1);
    }
    (gx, gy)
}

fn read_vals(input: &ImageData, scalars: &str) -> Vec<f64> {
    match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => {
            let mut buf = [0.0f64];
            (0..a.num_tuples())
                .map(|i| {
                    a.tuple_as_f64(i, &mut buf);
                    buf[0]
                })
                .collect()
        }
        _ => vec![],
    }
}

fn make_result(input: &ImageData, scalars: &str, data: Vec<f64>) -> ImageData {
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_img() -> ImageData {
        ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| {
                if x > 5.0 {
                    100.0
                } else {
                    0.0
                }
            },
        )
    }
    #[test]
    fn test_sobel() {
        let r = sobel_edge(&test_img(), "v");
        assert_eq!(r.dimensions(), [10, 10, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 10, &mut buf);
        assert!(buf[0] > 0.0); // edge at x=5
    }
    #[test]
    fn test_prewitt() {
        let r = prewitt_edge(&test_img(), "v");
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn test_roberts() {
        let r = roberts_edge(&test_img(), "v");
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn test_laplacian() {
        let r = laplacian_edge(&test_img(), "v");
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
}
