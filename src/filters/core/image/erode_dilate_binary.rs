use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Binary dilation on ImageData using a 3x3x3 ellipsoidal structuring element.
///
/// This is the binary specialization of VTK's `vtkImageDilateErode3D`:
/// 1-valued pixels dilate into neighboring 0-valued pixels.
pub fn binary_dilate(input: &ImageData, scalars: &str) -> ImageData {
    image_dilate_erode_3d(input, scalars, 1.0, 0.0, [3, 3, 3])
}

/// Binary erosion on ImageData using a 3x3x3 ellipsoidal structuring element.
///
/// This is the binary specialization of VTK's `vtkImageDilateErode3D`:
/// 0-valued pixels dilate into neighboring 1-valued pixels.
pub fn binary_erode(input: &ImageData, scalars: &str) -> ImageData {
    image_dilate_erode_3d(input, scalars, 0.0, 1.0, [3, 3, 3])
}

/// Dilate one value and erode another, following `vtkImageDilateErode3D`.
///
/// The input value is copied by default. Only pixels exactly equal to
/// `erode_value` are changed, and only when a pixel exactly equal to
/// `dilate_value` lies under the ellipsoidal kernel footprint.
pub fn image_dilate_erode_3d(
    input: &ImageData,
    scalars: &str,
    dilate_value: f64,
    erode_value: f64,
    kernel_size: [usize; 3],
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    let num_comps = arr.num_components();
    if n == 0 || num_comps == 0 || kernel_size.contains(&0) {
        return input.clone();
    }

    let mut values = vec![0.0; n * num_comps];
    let mut buf = vec![0.0; num_comps];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_comps;
        values[offset..offset + num_comps].copy_from_slice(&buf);
    }

    let mut result = values.clone();
    let kernel_middle = [kernel_size[0] / 2, kernel_size[1] / 2, kernel_size[2] / 2];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let tuple_idx = k * ny * nx + j * nx + i;
                for comp in 0..num_comps {
                    let center_idx = tuple_idx * num_comps + comp;
                    if values[center_idx] != erode_value {
                        continue;
                    }

                    'neighborhood: for kz in 0..kernel_size[2] {
                        let dz = kz as isize - kernel_middle[2] as isize;
                        let Some(kk) = k.checked_add_signed(dz) else {
                            continue;
                        };
                        if kk >= nz {
                            continue;
                        }
                        for ky in 0..kernel_size[1] {
                            let dy = ky as isize - kernel_middle[1] as isize;
                            let Some(jj) = j.checked_add_signed(dy) else {
                                continue;
                            };
                            if jj >= ny {
                                continue;
                            }
                            for kx in 0..kernel_size[0] {
                                if !ellipsoid_mask_value(kx, ky, kz, kernel_size) {
                                    continue;
                                }

                                let dx = kx as isize - kernel_middle[0] as isize;
                                let Some(ii) = i.checked_add_signed(dx) else {
                                    continue;
                                };
                                if ii >= nx {
                                    continue;
                                }

                                let idx = (kk * ny * nx + jj * nx + ii) * num_comps + comp;
                                if values[idx] == dilate_value {
                                    result[center_idx] = dilate_value;
                                    break 'neighborhood;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut img = input.clone();
    let mut new_attrs = DataSetAttributes::new();
    for idx in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(idx).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                num_comps,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
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
        let normalized = if radius[axis] != 0.0 {
            delta / radius[axis]
        } else if delta == 0.0 {
            0.0
        } else {
            f64::MAX
        };
        sum += normalized * normalized;
    }

    sum <= 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(
        nx: usize,
        ny: usize,
        nz: usize,
        ones: &[(usize, usize, usize)],
    ) -> ImageData {
        let n: usize = nx * ny * nz;
        let mut data: Vec<f64> = vec![0.0; n];
        for &(x, y, z) in ones {
            let idx: usize = z * ny * nx + y * nx + x;
            data[idx] = 1.0;
        }
        let mut img = ImageData::with_dimensions(nx, ny, nz);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Mask", data, 1)));
        img
    }

    fn get_value(img: &ImageData, x: usize, y: usize, z: usize) -> f64 {
        let dims = img.dimensions();
        let nx: usize = dims[0] as usize;
        let ny: usize = dims[1] as usize;
        let idx: usize = z * ny * nx + y * nx + x;
        let arr = img.point_data().get_array("Mask").unwrap();
        let mut buf: [f64; 1] = [0.0];
        arr.tuple_as_f64(idx, &mut buf);
        buf[0]
    }

    #[test]
    fn dilate_single_voxel() {
        let img = make_test_image(5, 5, 5, &[(2, 2, 2)]);
        let dilated = binary_dilate(&img, "Mask");

        // Center should still be 1
        assert!(get_value(&dilated, 2, 2, 2) > 0.5);
        // Neighbors should be 1
        assert!(get_value(&dilated, 1, 2, 2) > 0.5);
        assert!(get_value(&dilated, 3, 2, 2) > 0.5);
        // Far away should be 0
        assert!(get_value(&dilated, 0, 0, 0) < 0.5);
        assert!(get_value(&dilated, 4, 4, 4) < 0.5);
    }

    #[test]
    fn erode_removes_boundary() {
        // Fill a 3x3x3 block in a 5x5x5 image.
        let mut ones: Vec<(usize, usize, usize)> = Vec::new();
        for z in 1..4 {
            for y in 1..4 {
                for x in 1..4 {
                    ones.push((x, y, z));
                }
            }
        }
        let img = make_test_image(5, 5, 5, &ones);
        let eroded = binary_erode(&img, "Mask");

        assert!(get_value(&eroded, 2, 2, 2) > 0.5);
        assert!(get_value(&eroded, 1, 1, 1) < 0.5);
        assert!(get_value(&eroded, 3, 3, 3) < 0.5);
    }

    #[test]
    fn dilate_then_erode_closing() {
        // A single voxel: dilate then erode should reduce back but keep center
        let img = make_test_image(5, 5, 5, &[(2, 2, 2)]);
        let dilated = binary_dilate(&img, "Mask");
        let closed = binary_erode(&dilated, "Mask");
        // Center should still be 1 after closing
        assert!(get_value(&closed, 2, 2, 2) > 0.5);
    }

    #[test]
    fn vtk_style_erode_ignores_image_boundary() {
        let img = make_test_image(1, 1, 1, &[(0, 0, 0)]);
        let eroded = binary_erode(&img, "Mask");
        assert!(get_value(&eroded, 0, 0, 0) > 0.5);
    }

    #[test]
    fn vtk_style_exact_values() {
        let img = make_test_image(3, 1, 1, &[(0, 0, 0)]);
        let mut almost = img.clone();
        almost.point_data_mut().remove_array("Mask");
        almost
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Mask",
                vec![0.99, 0.0, 0.0],
                1,
            )));

        let dilated = binary_dilate(&almost, "Mask");
        assert!(get_value(&dilated, 1, 0, 0) < 0.5);
    }

    #[test]
    fn preserves_component_count() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Mask",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                2,
            )));

        let dilated = binary_dilate(&img, "Mask");
        let arr = dilated.point_data().get_array("Mask").unwrap();
        assert_eq!(arr.num_components(), 2);
        assert_eq!(arr.num_tuples(), 3);

        let mut tuple = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut tuple);
        assert_eq!(tuple, [1.0, 1.0]);
    }
}
