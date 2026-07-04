//! Unsharp mask sharpening for images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply unsharp mask: result = original + amount * (original - blurred).
pub fn unsharp_mask(input: &ImageData, scalars: &str, radius: usize, amount: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let n = arr.num_tuples();
    if dims.contains(&0) || n != dims[0] * dims[1] * dims[2] {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let r = radius as isize;
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let mut blurred = vec![0.0f64; n];

    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let mut sum = 0.0;
                let mut count = 0.0;
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let sx = ix as isize + dx;
                            let sy = iy as isize + dy;
                            let sz = iz as isize + dz;
                            if sx >= 0
                                && sx < nx as isize
                                && sy >= 0
                                && sy < ny as isize
                                && sz >= 0
                                && sz < nz as isize
                            {
                                sum += vals[sx as usize + sy as usize * nx + sz as usize * nx * ny];
                                count += 1.0;
                            }
                        }
                    }
                }
                blurred[ix + iy * nx + iz * nx * ny] = sum / count;
            }
        }
    }

    let data: Vec<f64> = (0..n)
        .map(|i| vals[i] + amount * (vals[i] - blurred[i]))
        .collect();

    ImageData::with_dimensions(nx, ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unsharp() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 4.0).abs() < 0.5 && (y - 4.0).abs() < 0.5 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let result = unsharp_mask(&img, "v", 1, 1.0);
        assert_eq!(result.dimensions(), [8, 8, 1]);
        // The peak should be enhanced
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(4 + 4 * 8, &mut buf);
        assert!(buf[0] > 100.0);
    }
}
