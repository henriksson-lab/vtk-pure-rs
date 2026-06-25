//! 3D morphological operations on ImageData: dilate, erode, open, close.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// 3D binary dilation with an ellipsoidal structuring element.
pub fn dilate_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    image_dilate_erode_3d(image, array_name, 1.0, 0.0, kernel_size_from_radius(radius))
}

/// 3D binary erosion with an ellipsoidal structuring element.
pub fn erode_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    image_dilate_erode_3d(image, array_name, 0.0, 1.0, kernel_size_from_radius(radius))
}

/// 3D morphological opening (erode then dilate).
pub fn open_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let eroded = erode_3d(image, array_name, radius);
    dilate_3d(&eroded, array_name, radius)
}

/// 3D morphological closing (dilate then erode).
pub fn close_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let dilated = dilate_3d(image, array_name, radius);
    erode_3d(&dilated, array_name, radius)
}

/// 3D morphological gradient (dilate - erode).
pub fn morphological_gradient_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let dilated = dilate_3d(image, array_name, radius);
    let eroded = erode_3d(image, array_name, radius);

    let d_arr = dilated.point_data().get_array(array_name).unwrap();
    let e_arr = eroded.point_data().get_array(array_name).unwrap();
    if d_arr.num_components() != 1 || e_arr.num_components() != 1 {
        return image.clone();
    }
    let n = d_arr.num_tuples();

    let mut grad = Vec::with_capacity(n);
    let mut db = [0.0f64];
    let mut eb = [0.0f64];
    for i in 0..n {
        d_arr.tuple_as_f64(i, &mut db);
        e_arr.tuple_as_f64(i, &mut eb);
        grad.push(db[0] - eb[0]);
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorphGradient",
            grad,
            1,
        )));
    result
}

/// Dilate one value and erode another, following `vtkImageDilateErode3D`.
///
/// The input value is copied by default. Only components exactly equal to
/// `erode_value` are changed, and only when an in-bounds component exactly
/// equal to `dilate_value` lies under the ellipsoidal kernel footprint.
pub fn image_dilate_erode_3d(
    image: &ImageData,
    array_name: &str,
    dilate_value: f64,
    erode_value: f64,
    kernel_size: [usize; 3],
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = nx * ny * nz;
    let num_components = arr.num_components();
    if n == 0 || num_components == 0 || kernel_size.contains(&0) {
        return image.clone();
    }

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    let mut output = values.clone();
    let kernel_middle = [kernel_size[0] / 2, kernel_size[1] / 2, kernel_size[2] / 2];

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let tuple_idx = z * ny * nx + y * nx + x;
                for component in 0..num_components {
                    let out_idx = tuple_idx * num_components + component;
                    if values[out_idx] != erode_value {
                        continue;
                    }

                    'neighborhood: for kz in 0..kernel_size[2] {
                        let dz = kz as isize - kernel_middle[2] as isize;
                        let Some(zz) = z.checked_add_signed(dz) else {
                            continue;
                        };
                        if zz >= nz {
                            continue;
                        }
                        for ky in 0..kernel_size[1] {
                            let dy = ky as isize - kernel_middle[1] as isize;
                            let Some(yy) = y.checked_add_signed(dy) else {
                                continue;
                            };
                            if yy >= ny {
                                continue;
                            }
                            for kx in 0..kernel_size[0] {
                                if !ellipsoid_mask_value(kx, ky, kz, kernel_size) {
                                    continue;
                                }

                                let dx = kx as isize - kernel_middle[0] as isize;
                                let Some(xx) = x.checked_add_signed(dx) else {
                                    continue;
                                };
                                if xx >= nx {
                                    continue;
                                }

                                let in_idx =
                                    (zz * ny * nx + yy * nx + xx) * num_components + component;
                                if values[in_idx] == dilate_value {
                                    output[out_idx] = dilate_value;
                                    break 'neighborhood;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result = image.clone();
    let mut point_data = DataSetAttributes::new();
    for i in 0..image.point_data().num_arrays() {
        let a = image.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            point_data.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                output.clone(),
                num_components,
            )));
        } else {
            point_data.add_array(a.clone());
        }
    }
    *result.point_data_mut() = point_data;
    result
}

fn kernel_size_from_radius(radius: usize) -> [usize; 3] {
    let size = radius.saturating_mul(2).saturating_add(1);
    [size, size, size]
}

fn ellipsoid_mask_value(x: usize, y: usize, z: usize, kernel_size: [usize; 3]) -> bool {
    let center = [
        (kernel_size[0] - 1) as f64 * 0.5,
        (kernel_size[1] - 1) as f64 * 0.5,
        (kernel_size[2] - 1) as f64 * 0.5,
    ];
    let radius = [
        kernel_size[0] as f64 * 0.5,
        kernel_size[1] as f64 * 0.5,
        kernel_size[2] as f64 * 0.5,
    ];
    let coords = [x as f64, y as f64, z as f64];
    let mut sum = 0.0;

    for axis in 0..3 {
        let delta = coords[axis] - center[axis];
        let normalized = delta / radius[axis];
        sum += normalized * normalized;
    }

    sum <= 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sphere_image() -> ImageData {
        ImageData::from_function(
            [20, 20, 20],
            [0.1, 0.1, 0.1],
            [0.0, 0.0, 0.0],
            "mask",
            |x, y, z| {
                if (x - 1.0).powi(2) + (y - 1.0).powi(2) + (z - 1.0).powi(2) < 0.25 {
                    1.0
                } else {
                    0.0
                }
            },
        )
    }

    #[test]
    fn dilate_grows() {
        let img = make_sphere_image();
        let dilated = dilate_3d(&img, "mask", 1);
        let orig_count = count_above(&img, "mask", 0.5);
        let new_count = count_above(&dilated, "mask", 0.5);
        assert!(new_count > orig_count);
    }

    #[test]
    fn erode_shrinks() {
        let img = make_sphere_image();
        let eroded = erode_3d(&img, "mask", 1);
        let orig_count = count_above(&img, "mask", 0.5);
        let new_count = count_above(&eroded, "mask", 0.5);
        assert!(new_count < orig_count);
    }

    #[test]
    fn open_close() {
        let img = make_sphere_image();
        let opened = open_3d(&img, "mask", 1);
        let closed = close_3d(&img, "mask", 1);
        assert!(count_above(&opened, "mask", 0.5) <= count_above(&img, "mask", 0.5));
        assert!(count_above(&closed, "mask", 0.5) >= count_above(&img, "mask", 0.5));
    }

    #[test]
    fn vtk_style_copies_unmatched_values() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0, 0.25, 0.0],
                1,
            )));

        let dilated = dilate_3d(&img, "mask", 1);
        let arr = dilated.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.25);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn vtk_style_processes_components_independently() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![
                    1.0, 0.0, //
                    0.0, 1.0, //
                    0.0, 0.0,
                ],
                2,
            )));

        let dilated = dilate_3d(&img, "mask", 1);
        let arr = dilated.point_data().get_array("mask").unwrap();
        assert_eq!(arr.num_components(), 2);

        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 1.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [0.0, 1.0]);
    }

    fn count_above(img: &ImageData, name: &str, thresh: f64) -> usize {
        let arr = img.point_data().get_array(name).unwrap();
        let mut c = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= thresh {
                c += 1;
            }
        }
        c
    }
}
