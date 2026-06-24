//! Image padding with various border modes.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Pad image with constant value.
pub fn pad_constant(input: &ImageData, scalars: &str, pad: usize, value: f64) -> ImageData {
    pad_generic(input, scalars, pad, |_, _, _, _| value)
}

/// Pad image by replicating edge pixels.
pub fn pad_replicate(input: &ImageData, scalars: &str, pad: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    pad_generic(input, scalars, pad, |ix, iy, _, _| {
        let cx = ix.clamp(0, nx as isize - 1) as usize;
        let cy = iy.clamp(0, ny as isize - 1) as usize;
        vals[cx + cy * nx]
    })
}

/// Pad image by reflecting at edges.
pub fn pad_reflect(input: &ImageData, scalars: &str, pad: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    pad_generic(input, scalars, pad, |ix, iy, _, _| {
        let cx = reflect_index(ix, nx);
        let cy = reflect_index(iy, ny);
        vals[cx + cy * nx]
    })
}

fn reflect_index(i: isize, n: usize) -> usize {
    if i < 0 {
        (-i - 1).min(n as isize - 1) as usize
    } else if i >= n as isize {
        (2 * n as isize - i - 1).max(0) as usize
    } else {
        i as usize
    }
}

fn pad_generic(
    input: &ImageData,
    scalars: &str,
    pad: usize,
    f: impl Fn(isize, isize, usize, usize) -> f64,
) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let new_nx = nx + 2 * pad;
    let new_ny = ny + 2 * pad;
    let p = pad as isize;
    let data: Vec<f64> = (0..new_nx * new_ny * nz)
        .map(|idx| {
            let iz = idx / (new_nx * new_ny);
            let rem = idx % (new_nx * new_ny);
            let iy = rem / new_nx;
            let ix = rem % new_nx;
            f(ix as isize - p, iy as isize - p, nx, ny)
        })
        .collect();
    ImageData::with_dimensions(new_nx, new_ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constant() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 5.0,
        );
        let r = pad_constant(&img, "v", 2, 0.0);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_replicate() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = pad_replicate(&img, "v", 1);
        assert_eq!(r.dimensions(), [6, 6, 1]);
    }
    #[test]
    fn test_reflect() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = pad_reflect(&img, "v", 1);
        assert_eq!(r.dimensions(), [6, 6, 1]);
    }
}
