//! Image slice operations: extract, insert, stack slices.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a 2D slice from a 3D ImageData at a given Z index.
pub fn extract_z_slice(image: &ImageData, array_name: &str, z_index: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    if z_index >= dims[2] {
        return ImageData::new();
    }
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..dims[0] * dims[1])
        .map(|i| {
            let idx = i + z_index * dims[0] * dims[1];
            if idx < arr.num_tuples() {
                arr.tuple_as_f64(idx, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[1], 1)
        .with_spacing([sp[0], sp[1], sp[2]])
        .with_origin([org[0], org[1], org[2] + z_index as f64 * sp[2]])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
}

/// Extract a 2D slice at a given Y index.
pub fn extract_y_slice(image: &ImageData, array_name: &str, y_index: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    if y_index >= dims[1] {
        return ImageData::new();
    }
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..dims[0] * dims[2])
        .map(|i| {
            let ix = i % dims[0];
            let iz = i / dims[0];
            let idx = ix + y_index * dims[0] + iz * dims[0] * dims[1];
            if idx < arr.num_tuples() {
                arr.tuple_as_f64(idx, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[2], 1)
        .with_spacing([sp[0], sp[2], sp[1]])
        .with_origin([org[0], org[2], org[1] + y_index as f64 * sp[1]])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
}

/// Extract a 2D slice at a given X index.
pub fn extract_x_slice(image: &ImageData, array_name: &str, x_index: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    if x_index >= dims[0] {
        return ImageData::new();
    }
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..dims[1] * dims[2])
        .map(|i| {
            let iy = i % dims[1];
            let iz = i / dims[1];
            let idx = x_index + iy * dims[0] + iz * dims[0] * dims[1];
            if idx < arr.num_tuples() {
                arr.tuple_as_f64(idx, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    ImageData::with_dimensions(dims[1], dims[2], 1)
        .with_spacing([sp[1], sp[2], sp[0]])
        .with_origin([org[1], org[2], org[0] + x_index as f64 * sp[0]])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
}

/// Maximum intensity projection along Z axis.
pub fn mip_z(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return ImageData::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..dims[0] * dims[1])
        .map(|i| {
            let ix = i % dims[0];
            let iy = i / dims[0];
            let mut max_v = f64::MIN;
            for iz in 0..dims[2] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    max_v = max_v.max(buf[0]);
                }
            }
            if max_v > f64::MIN {
                max_v
            } else {
                0.0
            }
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[1], 1)
        .with_spacing([sp[0], sp[1], 1.0])
        .with_origin([org[0], org[1], 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn z_slice() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, z| z,
        );
        let slice = extract_z_slice(&img, "v", 2);
        assert_eq!(slice.dimensions(), [5, 5, 1]);
        let arr = slice.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01);
    }
    #[test]
    fn y_slice() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let slice = extract_y_slice(&img, "v", 3);
        assert_eq!(slice.dimensions()[0], 5);
    }
    #[test]
    fn mip() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, z| z,
        );
        let result = mip_z(&img, "v");
        assert_eq!(result.dimensions(), [5, 5, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 0.01); // max Z = 4
    }
}
