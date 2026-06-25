//! Binary morphological operations (structuring element based).

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

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
    let (dilate_value, erode_value) = if dilate { (1.0, 0.0) } else { (0.0, 1.0) };
    image_dilate_erode_3d(
        input,
        scalars,
        dilate_value,
        erode_value,
        kernel_size_from_radius(radius),
    )
}

fn image_dilate_erode_3d(
    input: &ImageData,
    scalars: &str,
    dilate_value: f64,
    erode_value: f64,
    kernel_size: [usize; 3],
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = nx * ny * nz;
    let num_components = arr.num_components();
    if n == 0 || num_components == 0 || kernel_size.contains(&0) {
        return input.clone();
    }

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    let mut data = values.clone();
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
                                    data[out_idx] = dilate_value;
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
    let mut point_data = DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            point_data.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                data.clone(),
                num_components,
            )));
        } else {
            point_data.add_array(a.clone());
        }
    }
    *img.point_data_mut() = point_data;
    img
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

    #[test]
    fn test_non_binary_values_are_copied() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.25, 0.0],
                1,
            )));

        let result = binary_dilate(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.25);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn test_preserves_other_point_arrays() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![3.0, 4.0, 5.0],
                1,
            )));

        let result = binary_dilate(&img, "v", 1);
        assert!(result.point_data().get_array("other").is_some());
    }
}
