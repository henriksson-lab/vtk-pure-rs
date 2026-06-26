use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological dilation of a binary ImageData field.
pub fn image_dilate(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    image_dilate_erode_values(input, scalars, radius, 1.0, 0.0)
}

/// Morphological erosion of a binary ImageData field.
pub fn image_erode(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    image_dilate_erode_values(input, scalars, radius, 0.0, 1.0)
}

/// Dilate one value and erode another using an ellipsoidal footprint.
///
/// This follows `vtkImageDilateErode3D`: pixels equal to `erode_value` are
/// replaced with `dilate_value` when any in-bounds footprint pixel equals
/// `dilate_value`; all other pixels are copied unchanged.
pub fn image_dilate_erode_values(
    input: &ImageData,
    scalars: &str,
    radius: usize,
    dilate_value: f64,
    erode_value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let num_components = arr.num_components();
    if n == 0 || num_components == 0 {
        return input.clone();
    }
    let r = radius as i64;

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    let mut result = values.clone();
    let radius = r as f64 + 0.5;
    let radius2 = radius * radius;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let out_idx = k * ny * nx + j * nx + i;
                for comp in 0..num_components {
                    let out_comp_idx = out_idx * num_components + comp;
                    if values[out_comp_idx] != erode_value {
                        continue;
                    }

                    'neighborhood: for dk in -r..=r {
                        let kk = k as i64 + dk;
                        if kk < 0 || kk >= nz as i64 {
                            continue;
                        }
                        for dj in -r..=r {
                            let jj = j as i64 + dj;
                            if jj < 0 || jj >= ny as i64 {
                                continue;
                            }
                            for di in -r..=r {
                                let ii = i as i64 + di;
                                if ii < 0 || ii >= nx as i64 {
                                    continue;
                                }
                                let d2 =
                                    (di as f64).powi(2) + (dj as f64).powi(2) + (dk as f64).powi(2);
                                if d2 > radius2 {
                                    continue;
                                }
                                let in_idx = kk as usize * ny * nx + jj as usize * nx + ii as usize;
                                let in_comp_idx = in_idx * num_components + comp;
                                if values[in_comp_idx] == dilate_value {
                                    result[out_comp_idx] = dilate_value;
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
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                num_components,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_binary_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        values[12] = 1.0; // center pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    #[test]
    fn dilate_spreads() {
        let img = make_binary_image();
        let result = image_dilate(&img, "mask", 1);
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // Center should still be 1
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Neighbors should now be 1
        arr.tuple_as_f64(11, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(7, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn erode_shrinks() {
        // Start with a 3x3 block of 1s
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        for j in 1..4 {
            for i in 1..4 {
                values[j * 5 + i] = 1.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = image_erode(&img, "mask", 1);
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // Only center should remain
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Edge of block should be eroded
        arr.tuple_as_f64(6, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let img = make_binary_image();
        let result = image_dilate(&img, "nope", 1);
        assert!(result.point_data().get_array("mask").is_some());
    }

    #[test]
    fn vtk_style_processes_each_component_independently() {
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

        let result = image_dilate(&img, "mask", 1);
        let arr = result.point_data().get_array("mask").unwrap();
        assert_eq!(arr.num_components(), 2);

        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 1.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [0.0, 1.0]);
    }
}
