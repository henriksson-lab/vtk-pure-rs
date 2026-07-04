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
    if z_index >= dims[2] {
        return ImageData::new();
    }
    let org = image.point_from_ijk(0, 0, z_index);
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
        .with_origin(org)
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
    if y_index >= dims[1] {
        return ImageData::new();
    }
    let org = image.point_from_ijk(0, y_index, 0);
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
        .with_origin([org[0], org[2], org[1]])
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
    if x_index >= dims[0] {
        return ImageData::new();
    }
    let org = image.point_from_ijk(x_index, 0, 0);
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
        .with_origin([org[1], org[2], org[0]])
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
    let mut org = image.point_from_ijk(0, 0, 0);
    if dims[2] > 0 {
        org[2] += 0.5 * sp[2] * dims[2].saturating_sub(1) as f64;
    }
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..dims[0] * dims[1])
        .map(|i| {
            let ix = i % dims[0];
            let iy = i / dims[0];
            let mut max_v = f64::NEG_INFINITY;
            for iz in 0..dims[2] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    max_v = max_v.max(buf[0]);
                }
            }
            if max_v.is_finite() {
                max_v
            } else {
                0.0
            }
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[1], 1)
        .with_spacing([sp[0], sp[1], sp[2]])
        .with_origin(org)
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

    #[test]
    fn z_slice_origin_honors_extent() {
        let mut img = ImageData::with_dimensions(2, 2, 2);
        img.set_extent([10, 11, 20, 21, 30, 31]);
        img.set_spacing([2.0, 3.0, 4.0]);
        img.set_origin([1.0, 2.0, 3.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 8], 1)));

        let slice = extract_z_slice(&img, "v", 1);
        assert_eq!(slice.origin(), [21.0, 62.0, 127.0]);
    }

    #[test]
    fn mip_handles_negative_values() {
        let mut img = ImageData::with_dimensions(1, 1, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-10.0, -3.0, -7.0],
                1,
            )));

        let result = mip_z(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], -3.0);
    }
}
