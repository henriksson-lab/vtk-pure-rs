use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Binary dilation on ImageData using a 3x3x3 structuring element.
///
/// Any voxel neighboring a 1-valued voxel (in the 26-connected sense)
/// becomes 1 in the output. Works on the named scalar array which should
/// contain 0/1 values.
pub fn binary_dilate(input: &ImageData, scalars: &str) -> ImageData {
    binary_morphology(input, scalars, true)
}

/// Binary erosion on ImageData using a 3x3x3 structuring element.
///
/// A voxel remains 1 only if all 27 neighbors (including itself) in the
/// 3x3x3 neighborhood are 1. Works on the named scalar array with 0/1 values.
pub fn binary_erode(input: &ImageData, scalars: &str) -> ImageData {
    binary_morphology(input, scalars, false)
}

fn binary_morphology(input: &ImageData, scalars: &str, dilate: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    // Read input values
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let mut result: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                if dilate {
                    // Output is 1 if any neighbor is 1
                    let mut found: bool = false;
                    'outer: for dk in -1i64..=1 {
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
                                let idx: usize =
                                    kk as usize * ny * nx + jj as usize * nx + ii as usize;
                                if values[idx] > 0.5 {
                                    found = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                    result[k * ny * nx + j * nx + i] = if found { 1.0 } else { 0.0 };
                } else {
                    // Output is 1 only if all neighbors are 1
                    let mut all_one: bool = true;
                    'outer2: for dk in -1i64..=1 {
                        let kk: i64 = k as i64 + dk;
                        if kk < 0 || kk >= nz as i64 {
                            all_one = false;
                            break;
                        }
                        for dj in -1i64..=1 {
                            let jj: i64 = j as i64 + dj;
                            if jj < 0 || jj >= ny as i64 {
                                all_one = false;
                                break 'outer2;
                            }
                            for di in -1i64..=1 {
                                let ii: i64 = i as i64 + di;
                                if ii < 0 || ii >= nx as i64 {
                                    all_one = false;
                                    break 'outer2;
                                }
                                let idx: usize =
                                    kk as usize * ny * nx + jj as usize * nx + ii as usize;
                                if values[idx] < 0.5 {
                                    all_one = false;
                                    break 'outer2;
                                }
                            }
                        }
                    }
                    result[k * ny * nx + j * nx + i] = if all_one { 1.0 } else { 0.0 };
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
                1,
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
        // Fill a 3x3x3 block in a 5x5x5 image
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

        // Only center voxel (2,2,2) should survive (its full 3x3x3 neighborhood is 1)
        assert!(get_value(&eroded, 2, 2, 2) > 0.5);
        // Edge of the block should be eroded away
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
}
