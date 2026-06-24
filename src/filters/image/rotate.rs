//! Image rotation (90/180/270 degrees and arbitrary).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Rotate image 90 degrees clockwise.
pub fn rotate_90(input: &ImageData, scalars: &str) -> ImageData {
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
    let data: Vec<f64> = (0..nx * ny)
        .map(|idx| {
            let new_y = idx / ny;
            let new_x = idx % ny;
            let old_x = new_y;
            let old_y = ny - 1 - new_x;
            vals[old_x + old_y * nx]
        })
        .collect();
    ImageData::with_dimensions(ny, nx, dims[2])
        .with_spacing([input.spacing()[1], input.spacing()[0], input.spacing()[2]])
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Rotate image 180 degrees.
pub fn rotate_180(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let data: Vec<f64> = vals.iter().rev().cloned().collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Rotate image 270 degrees clockwise (= 90 counter-clockwise).
pub fn rotate_270(input: &ImageData, scalars: &str) -> ImageData {
    rotate_90(&rotate_90(&rotate_90(input, scalars), scalars), scalars)
}

/// Flip image horizontally (mirror along vertical axis).
pub fn flip_horizontal(input: &ImageData, scalars: &str) -> ImageData {
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
    let data: Vec<f64> = (0..nx * ny)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            vals[(nx - 1 - ix) + iy * nx]
        })
        .collect();
    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Flip image vertically (mirror along horizontal axis).
pub fn flip_vertical(input: &ImageData, scalars: &str) -> ImageData {
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
    let data: Vec<f64> = (0..nx * ny)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            vals[ix + (ny - 1 - iy) * nx]
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
    fn test_rotate_90() {
        let img = ImageData::from_function(
            [4, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = rotate_90(&img, "v");
        assert_eq!(r.dimensions(), [3, 4, 1]);
    }
    #[test]
    fn test_rotate_180() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = rotate_180(&img, "v");
        assert_eq!(r.dimensions(), [4, 4, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10); // last pixel becomes first
    }
    #[test]
    fn test_flip_h() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = flip_horizontal(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }
    #[test]
    fn test_flip_v() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let r = flip_vertical(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }
}
