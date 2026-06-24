//! Integral image (summed area table) for fast region queries.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute integral image (summed area table).
pub fn integral_image(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let mut sat = vec![0.0f64; nx * ny];

    for iy in 0..ny {
        for ix in 0..nx {
            let idx = ix + iy * nx;
            arr.tuple_as_f64(idx, &mut buf);
            let mut val = buf[0];
            if ix > 0 {
                val += sat[idx - 1];
            }
            if iy > 0 {
                val += sat[idx - nx];
            }
            if ix > 0 && iy > 0 {
                val -= sat[idx - nx - 1];
            }
            sat[idx] = val;
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "IntegralImage",
            sat,
            1,
        )))
}

/// Query sum over rectangle [x0,y0]-[x1,y1] from integral image.
pub fn query_rect_sum(integral: &ImageData, x0: usize, y0: usize, x1: usize, y1: usize) -> f64 {
    let arr = match integral.point_data().get_array("IntegralImage") {
        Some(a) => a,
        None => return 0.0,
    };
    let nx = integral.dimensions()[0];
    let mut buf = [0.0f64];
    let mut buf2 = [0.0f64];
    let mut get = |x: usize, y: usize| -> f64 {
        arr.tuple_as_f64(x + y * nx, &mut buf2);
        buf2[0]
    };
    let mut sum = get(x1, y1);
    if x0 > 0 {
        sum -= get(x0 - 1, y1);
    }
    if y0 > 0 {
        sum -= get(x1, y0 - 1);
    }
    if x0 > 0 && y0 > 0 {
        sum += get(x0 - 1, y0 - 1);
    }
    sum
}

/// Fast box mean using integral image.
pub fn box_mean_from_integral(integral: &ImageData, cx: usize, cy: usize, radius: usize) -> f64 {
    let dims = integral.dimensions();
    let x0 = if cx >= radius { cx - radius } else { 0 };
    let y0 = if cy >= radius { cy - radius } else { 0 };
    let x1 = (cx + radius).min(dims[0] - 1);
    let y1 = (cy + radius).min(dims[1] - 1);
    let area = ((x1 - x0 + 1) * (y1 - y0 + 1)) as f64;
    query_rect_sum(integral, x0, y0, x1, y1) / area
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_integral() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let ii = integral_image(&img, "v");
        let arr = ii.point_data().get_array("IntegralImage").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 3 * 4, &mut buf);
        assert!((buf[0] - 16.0).abs() < 1e-10); // sum of all 16 pixels
    }
    #[test]
    fn test_query() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let ii = integral_image(&img, "v");
        let s = query_rect_sum(&ii, 1, 1, 2, 2);
        assert!((s - 4.0).abs() < 1e-10); // 2x2 region
    }
}
