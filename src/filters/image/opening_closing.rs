use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological opening (erode then dilate) on a binary ImageData field.
///
/// Opening removes small bright features (noise) while preserving the overall shape.
/// Uses a 3x3x3 ellipsoidal footprint, matching vtkImageOpenClose3D with
/// OpenValue=1 and CloseValue=0.
pub fn morphological_opening(input: &ImageData, scalars: &str) -> ImageData {
    image_open_close(input, scalars, 1.0, 0.0)
}

/// Morphological closing (dilate then erode) on a binary ImageData field.
///
/// Closing fills small dark holes while preserving the overall shape.
/// Uses a 3x3x3 ellipsoidal footprint, matching vtkImageOpenClose3D with
/// OpenValue=0 and CloseValue=1.
pub fn morphological_closing(input: &ImageData, scalars: &str) -> ImageData {
    image_open_close(input, scalars, 0.0, 1.0)
}

fn image_open_close(
    input: &ImageData,
    scalars: &str,
    open_value: f64,
    close_value: f64,
) -> ImageData {
    let opened = dilate_erode(input, scalars, close_value, open_value);
    dilate_erode(&opened, scalars, open_value, close_value)
}

fn dilate_erode(
    input: &ImageData,
    scalars: &str,
    dilate_value: f64,
    erode_value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let num_components = arr.num_components();
    let n: usize = nx * ny * nz;
    if n == 0 || arr.num_tuples() < n {
        return input.clone();
    }

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    let mut result = values.clone();
    let footprint = ellipsoid_footprint([3, 3, 3]);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let idx = k * ny * nx + j * nx + i;
                for c in 0..num_components {
                    let out_idx = idx * num_components + c;
                    if values[out_idx] != erode_value {
                        continue;
                    }

                    'hood: for &(di, dj, dk) in &footprint {
                        let ii = i as isize + di;
                        let jj = j as isize + dj;
                        let kk = k as isize + dk;
                        if ii < 0
                            || jj < 0
                            || kk < 0
                            || ii >= nx as isize
                            || jj >= ny as isize
                            || kk >= nz as isize
                        {
                            continue;
                        }

                        let hood_idx = (kk as usize * ny * nx + jj as usize * nx + ii as usize)
                            * num_components
                            + c;
                        if values[hood_idx] == dilate_value {
                            result[out_idx] = dilate_value;
                            break 'hood;
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

fn ellipsoid_footprint(kernel_size: [usize; 3]) -> Vec<(isize, isize, isize)> {
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
    let middle = [
        (kernel_size[0] / 2) as isize,
        (kernel_size[1] / 2) as isize,
        (kernel_size[2] / 2) as isize,
    ];
    let mut offsets = Vec::new();
    for k in 0..kernel_size[2] {
        for j in 0..kernel_size[1] {
            for i in 0..kernel_size[0] {
                let d0 = (i as f64 - center[0]) / radius[0];
                let d1 = (j as f64 - center[1]) / radius[1];
                let d2 = (k as f64 - center[2]) / radius[2];
                if d0 * d0 + d1 * d1 + d2 * d2 <= 1.0 {
                    offsets.push((
                        i as isize - middle[0],
                        j as isize - middle[1],
                        k as isize - middle[2],
                    ));
                }
            }
        }
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image_with_dot() -> ImageData {
        // 5x5x1 image with a single bright pixel at center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        values[12] = 1.0; // center pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    fn make_image_with_hole() -> ImageData {
        // 5x5x1 image all 1s except a single dark pixel at center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![1.0f64; 25];
        values[12] = 0.0; // center hole
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    #[test]
    fn opening_removes_small_dot() {
        let img = make_image_with_dot();
        let result = morphological_opening(&img, "mask");
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // The single bright pixel should be removed by opening (erode kills it, dilate can't restore)
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 0.0, "opening should remove isolated bright pixel");
    }

    #[test]
    fn closing_fills_small_hole() {
        let img = make_image_with_hole();
        let result = morphological_closing(&img, "mask");
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // The single dark pixel should be filled by closing (dilate fills it, erode can't remove)
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0, "closing should fill isolated dark pixel");
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = morphological_opening(&img, "nonexistent");
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn other_labels_are_not_modified() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![2.0, 1.0, 0.0],
                1,
            )));

        let result = morphological_closing(&img, "mask");
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
    }
}
