use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a 2D slice from a 3D ImageData along the Z axis at index `k`.
///
/// Returns a new ImageData with dimensions (nx, ny, 1).
pub fn extract_slice_z(input: &ImageData, scalars: &str, k: usize) -> ImageData {
    extract_slice(input, scalars, 2, k)
}

/// Extract a 2D slice from a 3D ImageData along the Y axis at index `j`.
///
/// Returns a new ImageData with dimensions (nx, 1, nz).
pub fn extract_slice_y(input: &ImageData, scalars: &str, j: usize) -> ImageData {
    extract_slice(input, scalars, 1, j)
}

/// Extract a 2D slice from a 3D ImageData along the X axis at index `i`.
///
/// Returns a new ImageData with dimensions (1, ny, nz).
pub fn extract_slice_x(input: &ImageData, scalars: &str, i: usize) -> ImageData {
    extract_slice(input, scalars, 0, i)
}

fn extract_slice(input: &ImageData, scalars: &str, axis: usize, index: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return ImageData::with_dimensions(1, 1, 1),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    if nx == 0 || ny == 0 || nz == 0 {
        return ImageData::with_dimensions(0, 0, 0);
    }

    let ncomp: usize = arr.num_components();
    let clamped = index.min(dims[axis].saturating_sub(1));
    let capacity = match axis {
        0 => ny * nz,
        1 => nx * nz,
        _ => nx * ny,
    };
    let mut values: Vec<f64> = Vec::with_capacity(capacity * ncomp);
    let mut buf: Vec<f64> = vec![0.0; ncomp];

    match axis {
        0 => {
            for k in 0..nz {
                for j in 0..ny {
                    let src_idx: usize = k * ny * nx + j * nx + clamped;
                    arr.tuple_as_f64(src_idx, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
        }
        1 => {
            for k in 0..nz {
                for i in 0..nx {
                    let src_idx: usize = k * ny * nx + clamped * nx + i;
                    arr.tuple_as_f64(src_idx, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
        }
        _ => {
            for j in 0..ny {
                for i in 0..nx {
                    let src_idx: usize = clamped * ny * nx + j * nx + i;
                    arr.tuple_as_f64(src_idx, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
        }
    }

    let mut extent = input.extent();
    let absolute_index = extent[axis * 2] + clamped as i64;
    extent[axis * 2] = absolute_index;
    extent[axis * 2 + 1] = absolute_index;

    let mut img = ImageData::with_dimensions(0, 0, 0);
    img.set_extent(extent);
    img.set_spacing(input.spacing());
    img.set_origin(input.origin());
    img.with_point_array(AnyDataArray::F64(DataArray::from_vec(
        scalars, values, ncomp,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image() -> ImageData {
        let mut img = ImageData::with_dimensions(4, 3, 5);
        let n: usize = 4 * 3 * 5;
        let values: Vec<f64> = (0..n).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", values, 1)));
        img
    }

    #[test]
    fn extract_z_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_z(&img, "data", 2);
        assert_eq!(result.dimensions(), [4, 3, 1]);
        assert_eq!(result.extent(), [0, 3, 0, 2, 2, 2]);
        assert!(result.point_data().get_array("data").is_some());
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 4 * 3);
    }

    #[test]
    fn extract_y_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_y(&img, "data", 1);
        assert_eq!(result.dimensions(), [4, 1, 5]);
        assert_eq!(result.extent(), [0, 3, 1, 1, 0, 4]);
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 4 * 5);
    }

    #[test]
    fn extract_x_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_x(&img, "data", 0);
        assert_eq!(result.dimensions(), [1, 3, 5]);
        assert_eq!(result.extent(), [0, 0, 0, 2, 0, 4]);
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 3 * 5);
    }

    #[test]
    fn slice_keeps_origin() {
        let mut img = make_test_image();
        img.set_spacing([2.0, 3.0, 4.0]);
        img.set_origin([10.0, 20.0, 30.0]);

        let result = extract_slice_z(&img, "data", 2);
        assert_eq!(result.origin(), [10.0, 20.0, 30.0]);
        assert_eq!(result.point_from_ijk(0, 0, 0), [10.0, 20.0, 38.0]);
    }
}
