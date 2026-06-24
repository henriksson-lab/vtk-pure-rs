use crate::data::{AnyDataArray, DataArray, ImageData};

/// Remove small connected components (islands) from a binary ImageData.
///
/// Components with fewer than `min_size` voxels are set to 0.
/// Keeps only large connected regions. Adds a "Cleaned" array.
pub fn image_island_remove(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
    min_size: usize,
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

    let mut buf = [0.0f64];
    let mut fg = vec![false; n];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        fg[i] = buf[0] >= threshold;
    }

    // Label connected components
    let mut labels = vec![0u32; n];
    let mut label_sizes: Vec<usize> = vec![0]; // label 0 = background
    let mut current_label = 0u32;

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if !fg[pi] || labels[pi] != 0 {
                    continue;
                }
                current_label += 1;
                let mut stack = vec![(i, j, k)];
                let mut size = 0usize;
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
                    if labels[ci_idx] != 0 || !fg[ci_idx] {
                        continue;
                    }
                    labels[ci_idx] = current_label;
                    size += 1;
                    if ci > 0 {
                        stack.push((ci - 1, cj, ck));
                    }
                    if ci + 1 < nx {
                        stack.push((ci + 1, cj, ck));
                    }
                    if cj > 0 {
                        stack.push((ci, cj - 1, ck));
                    }
                    if cj + 1 < ny {
                        stack.push((ci, cj + 1, ck));
                    }
                    if ck > 0 {
                        stack.push((ci, cj, ck - 1));
                    }
                    if ck + 1 < nz {
                        stack.push((ci, cj, ck + 1));
                    }
                }
                label_sizes.push(size);
            }
        }
    }

    // Build cleaned mask
    let cleaned: Vec<f64> = labels
        .iter()
        .map(|&l| {
            if l > 0 && label_sizes[l as usize] >= min_size {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Cleaned", cleaned, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_small_island() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        // Big region [0-3], small island [5]
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0],
                1,
            )));

        let result = image_island_remove(&img, "m", 0.5, 3);
        let arr = result.point_data().get_array("Cleaned").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // big region kept
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 0.0); // small island removed
    }

    #[test]
    fn keep_all_large() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("m", vec![1.0; 5], 1)));

        let result = image_island_remove(&img, "m", 0.5, 3);
        let arr = result.point_data().get_array("Cleaned").unwrap();
        let mut buf = [0.0f64];
        for i in 0..5 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_island_remove(&img, "nope", 0.5, 1);
        assert!(result.point_data().get_array("Cleaned").is_none());
    }
}
