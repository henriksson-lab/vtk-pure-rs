use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract the largest connected component from a binary ImageData.
///
/// Labels all connected components and keeps only the one with the
/// most voxels. Adds "LargestComponent" binary array.
pub fn image_largest_component(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
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
    let mut label_sizes: Vec<usize> = vec![0]; // 0=background
    let mut current = 0u32;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if !fg[pi] || labels[pi] != 0 {
                    continue;
                }
                current += 1;
                let mut stack = vec![(i, j, k)];
                let mut size = 0;
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
                    if labels[ci_idx] != 0 || !fg[ci_idx] {
                        continue;
                    }
                    labels[ci_idx] = current;
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

    // VTK's vtkImageConnectivityFilter keeps the first region when sizes tie.
    let mut largest = 0u32;
    let mut largest_size = 0usize;
    for (label, &size) in label_sizes.iter().enumerate().skip(1) {
        if size > largest_size {
            largest = label as u32;
            largest_size = size;
        }
    }

    let result: Vec<f64> = labels
        .iter()
        .map(|&l| if l == largest { 1.0 } else { 0.0 })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LargestComponent",
            result,
            1,
        )));
    img
}

/// Count and return sizes of all connected components, sorted descending.
pub fn image_component_sizes(input: &ImageData, scalars: &str, threshold: f64) -> Vec<usize> {
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
    let mut visited = vec![false; n];
    let mut sizes = Vec::new();

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                if !fg[pi] || visited[pi] {
                    continue;
                }
                let mut stack = vec![(i, j, k)];
                let mut size = 0;
                while let Some((ci, cj, ck)) = stack.pop() {
                    let ci_idx = idx(ci, cj, ck);
                    if visited[ci_idx] || !fg[ci_idx] {
                        continue;
                    }
                    visited[ci_idx] = true;
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
                sizes.push(size);
            }
        }
    }

    sizes.sort_by(|a, b| b.cmp(a));
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_of_two() {
        let mut img = ImageData::with_dimensions(9, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                1,
            )));

        let result = image_largest_component(&img, "v", 0.5);
        let arr = result.point_data().get_array("LargestComponent").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // large component
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 0.0); // small component removed
    }

    #[test]
    fn component_sizes() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0],
                1,
            )));

        let sizes = image_component_sizes(&img, "v", 0.5);
        assert_eq!(sizes.len(), 2);
        assert_eq!(sizes[0], 3); // largest first
        assert_eq!(sizes[1], 2);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_largest_component(&img, "nope", 0.5);
        assert!(r.point_data().get_array("LargestComponent").is_none());
    }

    #[test]
    fn largest_tie_keeps_first_region() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 0.0, 1.0, 1.0],
                1,
            )));

        let result = image_largest_component(&img, "v", 0.5);
        let arr = result.point_data().get_array("LargestComponent").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
