//! Binary morphological operations (structuring element based).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Binary dilation with a square structuring element.
pub fn binary_dilate(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morph_op(input, scalars, radius, true)
}

/// Binary erosion with a square structuring element.
pub fn binary_erode(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morph_op(input, scalars, radius, false)
}

/// Binary opening (erode then dilate).
pub fn binary_open(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    binary_dilate(&binary_erode(input, scalars, radius), scalars, radius)
}

/// Binary closing (dilate then erode).
pub fn binary_close(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    binary_erode(&binary_dilate(input, scalars, radius), scalars, radius)
}

fn morph_op(input: &ImageData, scalars: &str, radius: usize, dilate: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();
    let r = radius as isize;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (nx * ny);
            let rem = idx % (nx * ny);
            let iy = rem / nx;
            let ix = rem % nx;
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
                            let v = vals[sx as usize + sy as usize * nx + sz as usize * nx * ny];
                            if dilate && v {
                                return 1.0;
                            }
                            if !dilate && !v {
                                return 0.0;
                            }
                        }
                    }
                }
            }
            if dilate {
                0.0
            } else {
                1.0
            }
        })
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
    fn test_dilate() {
        let img = ImageData::from_function(
            [7, 7, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 3.0).abs() < 0.5 && (y - 3.0).abs() < 0.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = binary_dilate(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(4 + 3 * 7, &mut buf); // neighbor of center
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn test_erode() {
        let img = ImageData::from_function(
            [7, 7, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 3.0).abs() < 1.5 && (y - 3.0).abs() < 1.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = binary_erode(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 3 * 7, &mut buf);
        assert_eq!(buf[0], 1.0); // center survives
    }
    #[test]
    fn test_open_close() {
        let img = ImageData::from_function(
            [9, 9, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 4.0).abs() < 2.5 && (y - 4.0).abs() < 2.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let opened = binary_open(&img, "v", 1);
        let closed = binary_close(&img, "v", 1);
        assert_eq!(opened.dimensions(), [9, 9, 1]);
        assert_eq!(closed.dimensions(), [9, 9, 1]);
    }
}
