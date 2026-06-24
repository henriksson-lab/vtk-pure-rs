use crate::data::{AnyDataArray, DataArray, ImageData};

/// Count the number of connected components in a binary ImageData using
/// 6-connectivity flood fill.
///
/// Voxels with scalar value >= 0.5 are considered foreground.
pub fn count_connected_components(input: &ImageData, scalars: &str) -> usize {
    let (_, count) = label_connected_components_inner(input, scalars);
    count
}

/// Label connected components in a binary ImageData using 6-connectivity
/// flood fill.
///
/// Voxels with scalar value >= 0.5 are considered foreground. Returns a new
/// ImageData with a "ComponentLabel" array where each foreground voxel is
/// assigned its component label (1-based). Background voxels are labeled 0.
pub fn label_connected_components(input: &ImageData, scalars: &str) -> ImageData {
    let (img, _) = label_connected_components_inner(input, scalars);
    img
}

fn label_connected_components_inner(input: &ImageData, scalars: &str) -> (ImageData, usize) {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return (input.clone(), 0),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    // Read scalar values
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let threshold: f64 = 0.5;
    let mut labels: Vec<i64> = vec![0; n];
    let mut current_label: i64 = 0;

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi: usize = idx(i, j, k);
                if values[pi] < threshold || labels[pi] != 0 {
                    continue;
                }

                // Flood fill with 6-connectivity
                current_label += 1;
                let mut stack: Vec<(usize, usize, usize)> = vec![(i, j, k)];
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx: usize = idx(ci, cj, ck);
                    if labels[ci_idx] != 0 || values[ci_idx] < threshold {
                        continue;
                    }
                    labels[ci_idx] = current_label;

                    // 6-connected neighbors
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
            }
        }
    }

    let label_f64: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ComponentLabel",
            label_f64,
            1,
        )));

    (img, current_label as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(nx: usize, ny: usize, nz: usize, vals: Vec<f64>) -> ImageData {
        let mut img = ImageData::with_dimensions(nx, ny, nz);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", vals, 1)));
        img
    }

    #[test]
    fn two_separate_blobs_3d() {
        // 5x1x1: [1,1,0,1,1] => 2 components
        let img = make_image(5, 1, 1, vec![1.0, 1.0, 0.0, 1.0, 1.0]);
        let count: usize = count_connected_components(&img, "mask");
        assert_eq!(count, 2);
    }

    #[test]
    fn labels_are_distinct() {
        let img = make_image(5, 1, 1, vec![1.0, 1.0, 0.0, 1.0, 1.0]);
        let result = label_connected_components(&img, "mask");
        let arr = result.point_data().get_array("ComponentLabel").unwrap();
        let mut buf: [f64; 1] = [0.0];

        arr.tuple_as_f64(0, &mut buf);
        let label_a: f64 = buf[0];
        arr.tuple_as_f64(3, &mut buf);
        let label_b: f64 = buf[0];

        assert!(label_a > 0.0);
        assert!(label_b > 0.0);
        assert!((label_a - label_b).abs() > 0.5); // different labels
    }

    #[test]
    fn single_connected_region() {
        let img = make_image(3, 3, 1, vec![1.0; 9]);
        let count: usize = count_connected_components(&img, "mask");
        assert_eq!(count, 1);
    }

    #[test]
    fn no_foreground() {
        let img = make_image(3, 3, 1, vec![0.0; 9]);
        let count: usize = count_connected_components(&img, "mask");
        assert_eq!(count, 0);
    }
}
