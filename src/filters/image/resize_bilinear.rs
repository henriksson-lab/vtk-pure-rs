//! Bilinear image resizing.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resize image to new dimensions using bilinear interpolation.
pub fn resize_bilinear(
    input: &ImageData,
    scalars: &str,
    new_nx: usize,
    new_ny: usize,
) -> ImageData {
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
            let sx = ix as f64 * (ox - 1) as f64 / (new_nx - 1).max(1) as f64;
            let sy = iy as f64 * (oy - 1) as f64 / (new_ny - 1).max(1) as f64;
            let x0 = (sx.floor() as usize).min(ox - 1);
            let x1 = (x0 + 1).min(ox - 1);
            let y0 = (sy.floor() as usize).min(oy - 1);
            let y1 = (y0 + 1).min(oy - 1);
            let fx = sx - x0 as f64;
            let fy = sy - y0 as f64;
            let v00 = vals[x0 + y0 * ox];
            let v10 = vals[x1 + y0 * ox];
            let v01 = vals[x0 + y1 * ox];
            let v11 = vals[x1 + y1 * ox];
            v00 * (1.0 - fx) * (1.0 - fy)
                + v10 * fx * (1.0 - fy)
                + v01 * (1.0 - fx) * fy
                + v11 * fx * fy
        })
        .collect();

    let sp = input.spacing();
    let new_sp = [
        sp[0] * ox as f64 / new_nx as f64,
        sp[1] * oy as f64 / new_ny as f64,
        sp[2],
    ];
    ImageData::with_dimensions(new_nx, new_ny, dims[2])
        .with_spacing(new_sp)
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Resize by a scale factor.
pub fn resize_by_factor(input: &ImageData, scalars: &str, factor: f64) -> ImageData {
    let dims = input.dimensions();
    let nx = (dims[0] as f64 * factor).round().max(1.0) as usize;
    let ny = (dims[1] as f64 * factor).round().max(1.0) as usize;
    resize_bilinear(input, scalars, nx, ny)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_upscale() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = resize_bilinear(&img, "v", 8, 8);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_downscale() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_bilinear(&img, "v", 4, 4);
        assert_eq!(r.dimensions(), [4, 4, 1]);
    }
    #[test]
    fn test_factor() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_by_factor(&img, "v", 0.5);
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
