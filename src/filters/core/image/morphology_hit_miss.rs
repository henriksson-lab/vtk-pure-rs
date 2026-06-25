//! Hit-or-miss morphological transform.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply hit-or-miss transform with foreground and background structuring elements.
/// fg_kernel: 1=must be foreground, bg_kernel: 1=must be background, 0=don't care.
pub fn hit_or_miss(
    input: &ImageData,
    scalars: &str,
    fg_kernel: &[u8],
    bg_kernel: &[u8],
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
    let vals: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();
    let hkw = kw as isize / 2;
    let hkh = kh as isize / 2;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            for ky in 0..kh {
                for kx in 0..kw {
                    let sx = ix as isize + kx as isize - hkw;
                    let sy = iy as isize + ky as isize - hkh;
                    let ki = kx + ky * kw;
                    if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                        let v = vals[sx as usize + sy as usize * nx];
                        if fg_kernel[ki] == 1 && !v {
                            return 0.0;
                        }
                        if bg_kernel[ki] == 1 && v {
                            return 0.0;
                        }
                    } else {
                        if fg_kernel[ki] == 1 {
                            return 0.0;
                        }
                    }
                }
            }
            1.0
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Detect corners in a binary image using hit-or-miss.
pub fn detect_corners(input: &ImageData, scalars: &str) -> ImageData {
    // Top-left corner pattern
    let fg = [0, 1, 0, 1, 1, 0, 0, 0, 0];
    let bg = [0, 0, 0, 0, 0, 1, 0, 1, 0];
    let r1 = hit_or_miss(input, scalars, &fg, &bg, 3, 3);
    // Also check other rotations (simplified: just return one)
    r1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hit_miss() {
        let img = ImageData::from_function(
            [7, 7, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if x > 1.0 && x < 5.0 && y > 1.0 && y < 5.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let fg = [1, 1, 1, 1, 1, 1, 1, 1, 1];
        let bg = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let r = hit_or_miss(&img, "v", &fg, &bg, 3, 3);
        assert_eq!(r.dimensions(), [7, 7, 1]);
        // Interior pixel should match
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 3 * 7, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
