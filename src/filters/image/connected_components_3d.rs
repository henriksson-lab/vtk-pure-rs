use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute volume (voxel count) of each connected component in 3D.
///
/// Returns a sorted Vec of (label, volume) pairs.
pub fn component_volumes_3d(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
) -> Vec<(usize, usize)> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let mut labels = vec![0u32; n];
    let mut current = 0u32;
    let mut volumes = Vec::new();

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if !fg[pi] || labels[pi] != 0 {
                    continue;
                }
                current += 1;
                let mut vol = 0usize;
                let mut stack = vec![(i, j, k)];
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
                    if labels[ci_idx] != 0 || !fg[ci_idx] {
                        continue;
                    }
                    labels[ci_idx] = current;
                    vol += 1;
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
                volumes.push((current as usize, vol));
            }
        }
    }

    volumes.sort_by(|a, b| b.1.cmp(&a.1)); // largest first
    volumes
}

/// Filter components by minimum volume.
pub fn filter_by_volume(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
    min_volume: usize,
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
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();
    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let mut labels = vec![0u32; n];
    let mut label_sizes = vec![0usize]; // index 0 = background
    let mut current = 0u32;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if !fg[pi] || labels[pi] != 0 {
                    continue;
                }
                current += 1;
                let mut vol = 0;
                let mut stack = vec![(i, j, k)];
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
                    if labels[ci_idx] != 0 || !fg[ci_idx] {
                        continue;
                    }
                    labels[ci_idx] = current;
                    vol += 1;
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
                label_sizes.push(vol);
            }
        }
    }

    let result: Vec<f64> = labels
        .iter()
        .map(|&l| {
            if l > 0 && label_sizes[l as usize] >= min_volume {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Filtered", result, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_components() {
        let mut img = ImageData::with_dimensions(9, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                1,
            )));

        let vols = component_volumes_3d(&img, "v", 0.5);
        assert_eq!(vols.len(), 2);
        assert_eq!(vols[0].1, 3); // larger
        assert_eq!(vols[1].1, 2);
    }

    #[test]
    fn filter_small() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0],
                1,
            )));

        let result = filter_by_volume(&img, "v", 0.5, 2);
        let arr = result.point_data().get_array("Filtered").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // big component kept
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // small component removed
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert!(component_volumes_3d(&img, "nope", 0.5).is_empty());
    }
}
