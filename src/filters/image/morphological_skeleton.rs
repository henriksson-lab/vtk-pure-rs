use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological skeletonization of a binary image.
///
/// Uses iterative erosion and subtraction: at each iteration, the image is
/// eroded, then opened (erode + dilate), and the difference between the
/// eroded image and the opening is added to the skeleton. Iteration continues
/// until the eroded image is empty.
///
/// The result has a "Skeleton" scalar array in point data with values 0.0 or 1.0.
pub fn morphological_skeleton(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    // Read input as binary
    let mut current: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        current[i] = if buf[0] > 0.5 { 1.0 } else { 0.0 };
    }

    let mut skeleton: Vec<f64> = vec![0.0; n];

    // Iterate until current image is empty
    loop {
        let eroded: Vec<f64> = erode_3d(&current, nx, ny, nz);

        // Check if eroded is empty
        let mut any_nonzero: bool = false;
        for &v in &eroded {
            if v > 0.5 {
                any_nonzero = true;
                break;
            }
        }
        if !any_nonzero {
            // Add remaining current to skeleton
            for i in 0..n {
                if current[i] > 0.5 {
                    skeleton[i] = 1.0;
                }
            }
            break;
        }

        // Opening = dilate(erode(current))
        let opened: Vec<f64> = dilate_3d(&eroded, nx, ny, nz);

        // Skeleton += eroded - opened (the residue)
        for i in 0..n {
            if eroded[i] > 0.5 && opened[i] < 0.5 {
                skeleton[i] = 1.0;
            }
        }

        current = eroded;
    }

    // Build output
    let mut output: ImageData = input.clone();
    let mut new_attrs: DataSetAttributes = DataSetAttributes::new();
    // Copy existing arrays
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        new_attrs.add_array(a.clone());
    }
    // Add skeleton array
    new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
        "Skeleton", skeleton, 1,
    )));
    *output.point_data_mut() = new_attrs;
    output
}

/// 3D binary erosion with a 3x3x3 structuring element (cross/full).
fn erode_3d(data: &[f64], nx: usize, ny: usize, nz: usize) -> Vec<f64> {
    let n: usize = nx * ny * nz;
    let mut result: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut all_set: bool = true;
                for dk in -1i64..=1 {
                    let kk: i64 = k as i64 + dk;
                    if kk < 0 || kk >= nz as i64 {
                        all_set = false;
                        break;
                    }
                    for dj in -1i64..=1 {
                        let jj: i64 = j as i64 + dj;
                        if jj < 0 || jj >= ny as i64 {
                            all_set = false;
                            break;
                        }
                        for di in -1i64..=1 {
                            let ii: i64 = i as i64 + di;
                            if ii < 0 || ii >= nx as i64 {
                                all_set = false;
                                break;
                            }
                            let idx: usize = kk as usize * ny * nx + jj as usize * nx + ii as usize;
                            if data[idx] < 0.5 {
                                all_set = false;
                                break;
                            }
                        }
                        if !all_set {
                            break;
                        }
                    }
                    if !all_set {
                        break;
                    }
                }
                if all_set {
                    result[k * ny * nx + j * nx + i] = 1.0;
                }
            }
        }
    }

    result
}

/// 3D binary dilation with a 3x3x3 structuring element.
fn dilate_3d(data: &[f64], nx: usize, ny: usize, nz: usize) -> Vec<f64> {
    let n: usize = nx * ny * nz;
    let mut result: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut any_set: bool = false;
                for dk in -1i64..=1 {
                    let kk: i64 = k as i64 + dk;
                    if kk < 0 || kk >= nz as i64 {
                        continue;
                    }
                    for dj in -1i64..=1 {
                        let jj: i64 = j as i64 + dj;
                        if jj < 0 || jj >= ny as i64 {
                            continue;
                        }
                        for di in -1i64..=1 {
                            let ii: i64 = i as i64 + di;
                            if ii < 0 || ii >= nx as i64 {
                                continue;
                            }
                            let idx: usize = kk as usize * ny * nx + jj as usize * nx + ii as usize;
                            if data[idx] > 0.5 {
                                any_set = true;
                                break;
                            }
                        }
                        if any_set {
                            break;
                        }
                    }
                    if any_set {
                        break;
                    }
                }
                if any_set {
                    result[k * ny * nx + j * nx + i] = 1.0;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cross_image() -> ImageData {
        // 7x7x1 image with a cross pattern
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values: Vec<f64> = vec![0.0; 49];
        // Horizontal bar at row 3
        for i in 1..6 {
            values[3 * 7 + i] = 1.0;
        }
        // Vertical bar at col 3
        for j in 1..6 {
            values[j * 7 + 3] = 1.0;
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Binary", values, 1)));
        img
    }

    #[test]
    fn skeleton_is_subset_of_original() {
        let img = make_cross_image();
        let result = morphological_skeleton(&img, "Binary");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        let orig = img.point_data().get_array("Binary").unwrap();
        let mut buf_s: [f64; 1] = [0.0];
        let mut buf_o: [f64; 1] = [0.0];
        for i in 0..49 {
            skel.tuple_as_f64(i, &mut buf_s);
            orig.tuple_as_f64(i, &mut buf_o);
            if buf_s[0] > 0.5 {
                assert!(
                    buf_o[0] > 0.5,
                    "skeleton pixel {} is set but original is not",
                    i
                );
            }
        }
    }

    #[test]
    fn skeleton_of_filled_block() {
        // A 5x5x1 filled block should have a skeleton at the center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = vec![1.0; 25];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        let result = morphological_skeleton(&img, "mask");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        // The center pixel (2,2) should be in the skeleton
        let mut buf: [f64; 1] = [0.0];
        skel.tuple_as_f64(2 * 5 + 2, &mut buf);
        assert!(buf[0] > 0.5, "center should be in skeleton");
    }

    #[test]
    fn empty_image_produces_empty_skeleton() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = vec![0.0; 25];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        let result = morphological_skeleton(&img, "mask");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        let mut buf: [f64; 1] = [0.0];
        for i in 0..25 {
            skel.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 0.5, "empty image should have empty skeleton");
        }
    }
}
