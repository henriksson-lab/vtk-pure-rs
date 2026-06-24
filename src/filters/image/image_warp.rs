//! Image warping and geometric transforms.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Warp image using a displacement field (2-component array).
pub fn warp_by_field(input: &ImageData, scalars: &str, field_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let field = match input.point_data().get_array(field_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut sbuf = [0.0f64];
    let mut fbuf = [0.0f64; 2];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut sbuf);
            sbuf[0]
        })
        .collect();

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            field.tuple_as_f64(idx, &mut fbuf);
            let sx = (ix as f64 + fbuf[0]).round() as isize;
            let sy = (iy as f64 + fbuf[1]).round() as isize;
            if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                vals[sx as usize + sy as usize * nx]
            } else {
                0.0
            }
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Apply polar coordinate transform (Cartesian to polar).
pub fn cartesian_to_polar(input: &ImageData, scalars: &str) -> ImageData {
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
    let cx = nx as f64 / 2.0;
    let cy = ny as f64 / 2.0;
    let max_r = cx.min(cy);

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            // In polar output: x = angle, y = radius
            let angle = ix as f64 / nx as f64 * 2.0 * std::f64::consts::PI;
            let radius = iy as f64 / ny as f64 * max_r;
            let sx = (cx + radius * angle.cos()).round() as isize;
            let sy = (cy + radius * angle.sin()).round() as isize;
            if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                vals[sx as usize + sy as usize * nx]
            } else {
                0.0
            }
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_polar() {
        let img = ImageData::from_function(
            [16, 16, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                let dx = x - 8.0;
                let dy = y - 8.0;
                (dx * dx + dy * dy).sqrt()
            },
        );
        let r = cartesian_to_polar(&img, "v");
        assert_eq!(r.dimensions(), [16, 16, 1]);
    }
}
