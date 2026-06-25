use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute cumulative sum along the X axis of an ImageData.
///
/// Each voxel becomes the sum of all values at or before it in X.
/// Useful for integral images and fast area queries.
pub fn image_cumulative_sum_x(input: &ImageData, scalars: &str) -> ImageData {
    cumulative_axis(input, scalars, 0, "CumulativeSumX")
}

/// Compute cumulative sum along the Y axis.
pub fn image_cumulative_sum_y(input: &ImageData, scalars: &str) -> ImageData {
    cumulative_axis(input, scalars, 1, "CumulativeSumY")
}

/// Compute cumulative sum along the Z axis.
pub fn image_cumulative_sum_z(input: &ImageData, scalars: &str) -> ImageData {
    cumulative_axis(input, scalars, 2, "CumulativeSumZ")
}

fn cumulative_axis(input: &ImageData, scalars: &str, axis: usize, output: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    match axis {
        0 => {
            for k in 0..nz {
                for j in 0..ny {
                    for i in 1..nx {
                        values[idx(i, j, k)] += values[idx(i - 1, j, k)];
                    }
                }
            }
        }
        1 => {
            for k in 0..nz {
                for j in 1..ny {
                    for i in 0..nx {
                        values[idx(i, j, k)] += values[idx(i, j - 1, k)];
                    }
                }
            }
        }
        _ => {
            for k in 1..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        values[idx(i, j, k)] += values[idx(i, j, k - 1)];
                    }
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, 1)));
    img
}

/// Compute the full integral image (summed area table) for 2D ImageData.
/// Each pixel becomes the sum of all pixels above and to the left.
pub fn image_integral(input: &ImageData, scalars: &str) -> ImageData {
    let r1 = image_cumulative_sum_x(input, scalars);
    image_cumulative_sum_y(&r1, "CumulativeSumX")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumsum_x() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_cumulative_sum_x(&img, "v");
        let arr = result.point_data().get_array("CumulativeSumX").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 10.0);
    }

    #[test]
    fn cumsum_y() {
        let mut img = ImageData::with_dimensions(1, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = image_cumulative_sum_y(&img, "v");
        let arr = result.point_data().get_array("CumulativeSumY").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 6.0);
    }

    #[test]
    fn integral_image() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 1.0],
                1,
            )));

        let result = image_integral(&img, "v");
        let arr = result.point_data().get_array("CumulativeSumY").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 4.0); // sum of all
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_cumulative_sum_x(&img, "nope");
        assert!(r.point_data().get_array("CumulativeSumX").is_none());
    }
}
