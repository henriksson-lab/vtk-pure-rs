//! Image cropping and rotation operations.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Crop an ImageData to a sub-extent.
pub fn crop_extent(
    image: &ImageData,
    array_name: &str,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
) -> ImageData {
    let dims = image.dimensions();
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let x0 = x_range.0.min(dims[0]);
    let x1 = x_range.1.min(dims[0]);
    let y0 = y_range.0.min(dims[1]);
    let y1 = y_range.1.min(dims[1]);
    let z0 = z_range.0.min(dims[2]);
    let z1 = z_range.1.min(dims[2]);
    if x0 >= x1 || y0 >= y1 || z0 >= z1 {
        return ImageData::new();
    }

    let new_dims = [x1 - x0, y1 - y0, z1 - z0];
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2] * num_components);
    for iz in z0..z1 {
        for iy in y0..y1 {
            for ix in x0..x1 {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    data.extend_from_slice(&buf);
                } else {
                    data.extend(std::iter::repeat(0.0).take(num_components));
                }
            }
        }
    }

    let sp = image.spacing();
    let origin = image.origin();
    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin([
            origin[0] + x0 as f64 * sp[0],
            origin[1] + y0 as f64 * sp[1],
            origin[2] + z0 as f64 * sp[2],
        ])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            data,
            num_components,
        )))
}

/// Rotate a 2D image by 90 degrees clockwise.
pub fn rotate_90_cw(image: &ImageData, array_name: &str) -> ImageData {
    let dims = image.dimensions();
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 {
        return ImageData::new();
    }
    let new_dims = [dims[1], dims[0], dims[2]]; // swap X and Y
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2] * num_components);
    for iz in 0..dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                // 90° CW: new(x,y) = old(y, width-1-x)
                let old_x = iy;
                let old_y = new_dims[0] - 1 - ix;
                let old_idx = old_x + old_y * dims[0] + iz * dims[0] * dims[1];
                if old_idx < arr.num_tuples() {
                    arr.tuple_as_f64(old_idx, &mut buf);
                    data.extend_from_slice(&buf);
                } else {
                    data.extend(std::iter::repeat(0.0).take(num_components));
                }
            }
        }
    }

    let sp = image.spacing();
    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing([sp[1], sp[0], sp[2]])
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            data,
            num_components,
        )))
}

/// Flip a 2D/3D image along an axis.
pub fn flip_axis(image: &ImageData, array_name: &str, axis: usize) -> ImageData {
    let dims = image.dimensions();
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 {
        return ImageData::new();
    }
    let n = dims[0] * dims[1] * dims[2];
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(n * num_components);

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let (fx, fy, fz) = match axis {
                    0 => (dims[0] - 1 - ix, iy, iz),
                    1 => (ix, dims[1] - 1 - iy, iz),
                    _ => (ix, iy, dims[2] - 1 - iz),
                };
                let idx = fx + fy * dims[0] + fz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    data.extend_from_slice(&buf);
                } else {
                    data.extend(std::iter::repeat(0.0).take(num_components));
                }
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            data,
            num_components,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crop() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = crop_extent(&img, "v", (2, 8), (3, 7), (0, 1));
        assert_eq!(result.dimensions(), [6, 4, 1]);
    }
    #[test]
    fn rotate() {
        let img = ImageData::from_function(
            [4, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y * 10.0,
        );
        let result = rotate_90_cw(&img, "v");
        assert_eq!(result.dimensions(), [3, 4, 1]);
    }
    #[test]
    fn flip() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = flip_axis(&img, "v", 0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 0.01); // first voxel should now be the last x value
    }
}
