use crate::data::{AnyDataArray, DataArray, ImageData};

/// Morphological thinning (skeletonization) of a 2D binary ImageData.
///
/// Iteratively removes boundary pixels that don't disconnect the
/// foreground until only a 1-pixel-wide skeleton remains.
/// Works on XY slices (nz=1). Adds "Skeleton" array.
pub fn image_thin(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;

    let mut buf = [0.0f64];
    let mut img: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();

    // Zhang-Suen thinning (simplified)
    let mut changed = true;
    while changed {
        changed = false;
        for pass in 0..2 {
            let mut to_remove = vec![false; n];
            for j in 1..ny - 1 {
                for i in 1..nx - 1 {
                    if !img[j * nx + i] {
                        continue;
                    }
                    // 8-neighbors
                    let p2 = img[(j - 1) * nx + i] as u8;
                    let p3 = img[(j - 1) * nx + i + 1] as u8;
                    let p4 = img[j * nx + i + 1] as u8;
                    let p5 = img[(j + 1) * nx + i + 1] as u8;
                    let p6 = img[(j + 1) * nx + i] as u8;
                    let p7 = img[(j + 1) * nx + i - 1] as u8;
                    let p8 = img[j * nx + i - 1] as u8;
                    let p9 = img[(j - 1) * nx + i - 1] as u8;

                    let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                    if b < 2 || b > 6 {
                        continue;
                    }

                    // Count 0->1 transitions
                    let seq = [p2, p3, p4, p5, p6, p7, p8, p9, p2];
                    let a: u8 = (0..8)
                        .map(|k| if seq[k] == 0 && seq[k + 1] == 1 { 1 } else { 0 })
                        .sum();
                    if a != 1 {
                        continue;
                    }

                    if pass == 0 {
                        if p2 * p4 * p6 != 0 {
                            continue;
                        }
                        if p4 * p6 * p8 != 0 {
                            continue;
                        }
                    } else {
                        if p2 * p4 * p8 != 0 {
                            continue;
                        }
                        if p2 * p6 * p8 != 0 {
                            continue;
                        }
                    }
                    to_remove[j * nx + i] = true;
                }
            }

            for k in 0..n {
                if to_remove[k] {
                    img[k] = false;
                    changed = true;
                }
            }
        }
    }

    let skeleton: Vec<f64> = img.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();
    let mut out = input.clone();
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Skeleton", skeleton, 1,
        )));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thin_rectangle() {
        let mut img = ImageData::with_dimensions(10, 5, 1);
        let mut values = vec![0.0; 50];
        for j in 1..4 {
            for i in 1..9 {
                values[j * 10 + i] = 1.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_thin(&img, "v", 0.5);
        let arr = result.point_data().get_array("Skeleton").unwrap();
        let mut buf = [0.0f64];
        let mut count = 0;
        for i in 0..50 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        // Skeleton should have fewer pixels than the rectangle
        assert!(count < 24); // original has 24 foreground pixels
        assert!(count > 0);
    }

    #[test]
    fn single_pixel_preserved() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0; 25];
        values[12] = 1.0; // single pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_thin(&img, "v", 0.5);
        let arr = result.point_data().get_array("Skeleton").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let r = image_thin(&img, "nope", 0.5);
        assert!(r.point_data().get_array("Skeleton").is_none());
    }
}
