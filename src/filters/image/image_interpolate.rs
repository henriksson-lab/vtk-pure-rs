//! Image interpolation methods (bicubic, nearest, Lanczos).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resize image using nearest-neighbor interpolation.
pub fn resize_nearest(input: &ImageData, scalars: &str, new_nx: usize, new_ny: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (ox, oy) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let data: Vec<f64> = (0..new_nx * new_ny)
        .map(|idx| {
            let iy = idx / new_nx;
            let ix = idx % new_nx;
            let sx = (ix as f64 * ox as f64 / new_nx as f64).round() as usize;
            let sy = (iy as f64 * oy as f64 / new_ny as f64).round() as usize;
            vals[sx.min(ox - 1) + sy.min(oy - 1) * ox]
        })
        .collect();
    ImageData::with_dimensions(new_nx, new_ny, dims[2])
        .with_spacing([
            input.spacing()[0] * ox as f64 / new_nx as f64,
            input.spacing()[1] * oy as f64 / new_ny as f64,
            input.spacing()[2],
        ])
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Resize image using bicubic interpolation.
pub fn resize_bicubic(input: &ImageData, scalars: &str, new_nx: usize, new_ny: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (ox, oy) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let get = |x: isize, y: isize| -> f64 {
        let cx = x.clamp(0, ox as isize - 1) as usize;
        let cy = y.clamp(0, oy as isize - 1) as usize;
        vals[cx + cy * ox]
    };
    let data: Vec<f64> = (0..new_nx * new_ny)
        .map(|idx| {
            let iy = idx / new_nx;
            let ix = idx % new_nx;
            let fx = ix as f64 * (ox - 1) as f64 / (new_nx - 1).max(1) as f64;
            let fy = iy as f64 * (oy - 1) as f64 / (new_ny - 1).max(1) as f64;
            let x0 = fx.floor() as isize;
            let y0 = fy.floor() as isize;
            let dx = fx - x0 as f64;
            let dy = fy - y0 as f64;
            let mut sum = 0.0;
            for j in -1..=2isize {
                let wy = cubic_weight(dy - j as f64);
                for i in -1..=2isize {
                    let wx = cubic_weight(dx - i as f64);
                    sum += get(x0 + i, y0 + j) * wx * wy;
                }
            }
            sum
        })
        .collect();
    ImageData::with_dimensions(new_nx, new_ny, dims[2])
        .with_spacing([
            input.spacing()[0] * ox as f64 / new_nx as f64,
            input.spacing()[1] * oy as f64 / new_ny as f64,
            input.spacing()[2],
        ])
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn cubic_weight(t: f64) -> f64 {
    let t = t.abs();
    if t <= 1.0 {
        (1.5 * t - 2.5) * t * t + 1.0
    } else if t <= 2.0 {
        ((-0.5 * t + 2.5) * t - 4.0) * t + 2.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nearest() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_nearest(&img, "v", 8, 8);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_bicubic() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = resize_bicubic(&img, "v", 16, 16);
        assert_eq!(r.dimensions(), [16, 16, 1]);
    }
}
