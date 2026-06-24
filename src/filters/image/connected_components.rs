use crate::data::{AnyDataArray, DataArray, ImageData};

/// Label connected components in a binary ImageData field.
///
/// Uses flood-fill to assign a unique integer label to each connected
/// region where scalar >= threshold. Adds a "Labels" scalar array.
/// Returns the number of components found.
pub fn image_connected_components(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
) -> (ImageData, usize) {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return (input.clone(), 0),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let mut labels = vec![0i64; n];
    let mut current_label = 0i64;

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if values[pi] < threshold || labels[pi] != 0 {
                    continue;
                }

                // Flood fill
                current_label += 1;
                let mut stack = vec![(i, j, k)];
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
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
            "Labels", label_f64, 1,
        )));

    (img, current_label as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_blobs() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        // Two separate blobs: [1,1,0,1,1]
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0, 1.0, 0.0, 1.0, 1.0],
                1,
            )));

        let (result, count) = image_connected_components(&img, "mask", 0.5);
        assert_eq!(count, 2);
        let arr = result.point_data().get_array("Labels").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let label_a = buf[0];
        arr.tuple_as_f64(3, &mut buf);
        let label_b = buf[0];
        assert!(label_a > 0.0);
        assert!(label_b > 0.0);
        assert_ne!(label_a, label_b);
    }

    #[test]
    fn single_component() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0; 9],
                1,
            )));

        let (_, count) = image_connected_components(&img, "mask", 0.5);
        assert_eq!(count, 1);
    }

    #[test]
    fn no_foreground() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![0.0; 9],
                1,
            )));

        let (_, count) = image_connected_components(&img, "mask", 0.5);
        assert_eq!(count, 0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let (_, count) = image_connected_components(&img, "nope", 0.5);
        assert_eq!(count, 0);
    }
}
