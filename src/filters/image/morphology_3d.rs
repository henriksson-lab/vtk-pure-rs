//! 3D morphological operations on ImageData: dilate, erode, open, close.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D binary dilation with a spherical structuring element.
pub fn dilate_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    morphology_op(image, array_name, radius, true)
}

/// 3D binary erosion with a spherical structuring element.
pub fn erode_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    morphology_op(image, array_name, radius, false)
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

fn morphology_op(image: &ImageData, array_name: &str, radius: usize, is_dilate: bool) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let r = radius as i64;

    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    let mut output = vec![0.0f64; n];
    let threshold = 0.5;

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let mut found = false;

                'search: for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r * r {
                                continue;
                            }
                            let nx = ix as i64 + dx;
                            let ny = iy as i64 + dy;
                            let nz = iz as i64 + dz;
                            if nx < 0 || ny < 0 || nz < 0 {
                                continue;
                            }
                            let nx = nx as usize;
                            let ny = ny as usize;
                            let nz = nz as usize;
                            if nx >= dims[0] || ny >= dims[1] || nz >= dims[2] {
                                continue;
                            }
                            let ni = nx + ny * dims[0] + nz * dims[0] * dims[1];
                            if is_dilate {
                                if values[ni] >= threshold {
                                    found = true;
                                    break 'search;
                                }
                            } else {
                                if values[ni] < threshold {
                                    found = true;
                                    break 'search;
                                }
                            }
                        }
                    }
                }

                output[idx] = if is_dilate {
                    if found {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    if found {
                        0.0
                    } else {
                        if values[idx] >= threshold {
                            1.0
                        } else {
                            0.0
                        }
                    }
                };
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
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
