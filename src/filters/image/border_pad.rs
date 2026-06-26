//! Image padding with various border modes.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Pad image with constant value.
pub fn pad_constant(input: &ImageData, scalars: &str, pad: usize, value: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    pad_generic(input, scalars, pad, |ix, iy, iz| {
        if ix < 0 || iy < 0 || iz < 0 || ix >= nx as isize || iy >= ny as isize || iz >= nz as isize
        {
            value
        } else {
            vals[ix as usize + iy as usize * nx + iz as usize * nx * ny]
        }
    })
}

/// Pad image by replicating edge pixels.
pub fn pad_replicate(input: &ImageData, scalars: &str, pad: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    pad_generic(input, scalars, pad, |ix, iy, iz| {
        let cx = ix.clamp(0, nx as isize - 1) as usize;
        let cy = iy.clamp(0, ny as isize - 1) as usize;
        let cz = iz.clamp(0, nz as isize - 1) as usize;
        vals[cx + cy * nx + cz * nx * ny]
    })
}

/// Pad image by reflecting at edges.
pub fn pad_reflect(input: &ImageData, scalars: &str, pad: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    pad_generic(input, scalars, pad, |ix, iy, iz| {
        let cx = reflect_index(ix, nx);
        let cy = reflect_index(iy, ny);
        let cz = reflect_index(iz, nz);
        vals[cx + cy * nx + cz * nx * ny]
    })
}

fn reflect_index(i: isize, n: usize) -> usize {
    if n <= 1 {
        return 0;
    }

    let n = n as isize;
    let period = 2 * n;
    let mut idx = i % period;
    if idx < 0 {
        idx += period;
    }
    if idx >= n {
        (period - idx - 1) as usize
    } else {
        idx as usize
    }
}

fn pad_generic(
    input: &ImageData,
    scalars: &str,
    pad: usize,
    f: impl Fn(isize, isize, isize) -> f64,
) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let new_nx = nx + 2 * pad;
    let new_ny = ny + 2 * pad;
    let new_nz = nz + 2 * pad;
    let p = pad as isize;
    let data: Vec<f64> = (0..new_nx * new_ny * new_nz)
        .map(|idx| {
            let iz = idx / (new_nx * new_ny);
            let rem = idx % (new_nx * new_ny);
            let iy = rem / new_nx;
            let ix = rem % new_nx;
            f(ix as isize - p, iy as isize - p, iz as isize - p)
        })
        .collect();
    let origin = input.origin();
    let spacing = input.spacing();
    let new_origin = [
        origin[0] - pad as f64 * spacing[0],
        origin[1] - pad as f64 * spacing[1],
        origin[2] - pad as f64 * spacing[2],
    ];
    ImageData::with_dimensions(new_nx, new_ny, new_nz)
        .with_spacing(spacing)
        .with_origin(new_origin)
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
        assert_eq!(r.dimensions(), [8, 8, 5]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(2 + 2 * 8 + 2 * 8 * 8, &mut buf);
        assert_eq!(buf[0], 5.0);
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
        assert_eq!(r.dimensions(), [6, 6, 3]);
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
        assert_eq!(r.dimensions(), [6, 6, 3]);
    }

    #[test]
    fn test_reflect_large_pad_repeats() {
        assert_eq!(reflect_index(-5, 3), 1);
        assert_eq!(reflect_index(-4, 3), 2);
        assert_eq!(reflect_index(3, 3), 2);
        assert_eq!(reflect_index(4, 3), 1);
        assert_eq!(reflect_index(5, 3), 0);
    }
}
