use crate::data::{AnyDataArray, DataArray, ImageData};

/// Simple watershed segmentation on ImageData.
///
/// Starting from local minima (voxels whose value is <= all neighbors),
/// iteratively assigns each voxel to the nearest basin by flooding in
/// order of increasing scalar value. Adds a "WatershedLabel" point data
/// array with integer labels (starting from 1). Voxels that remain
/// unlabeled get label 0.
pub fn watershed_segment(input: &ImageData, scalars: &str) -> ImageData {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n: usize = nx * ny * nz;

    if n == 0 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let idx = |x: usize, y: usize, z: usize| -> usize { z * ny * nx + y * nx + x };

    // Find neighbors
    let neighbors_of = |pi: usize| -> Vec<usize> {
        let x: usize = pi % nx;
        let y: usize = (pi / nx) % ny;
        let z: usize = pi / (ny * nx);
        let mut nb = Vec::with_capacity(6);
        if x > 0 {
            nb.push(idx(x - 1, y, z));
        }
        if x + 1 < nx {
            nb.push(idx(x + 1, y, z));
        }
        if y > 0 {
            nb.push(idx(x, y - 1, z));
        }
        if y + 1 < ny {
            nb.push(idx(x, y + 1, z));
        }
        if z > 0 {
            nb.push(idx(x, y, z - 1));
        }
        if z + 1 < nz {
            nb.push(idx(x, y, z + 1));
        }
        nb
    };

    // Find local minima as seeds
    let mut labels = vec![0i64; n];
    let mut label_counter: i64 = 0;

    #[derive(PartialEq)]
    struct S(f64, usize);
    impl Eq for S {}
    impl PartialOrd for S {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for S {
        fn cmp(&self, o: &Self) -> Ordering {
            // Min-heap: reverse comparison
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let mut heap = BinaryHeap::new();

    for i in 0..n {
        let nbs = neighbors_of(i);
        let is_minimum = nbs.iter().all(|&ni| values[i] <= values[ni]);
        if is_minimum {
            label_counter += 1;
            labels[i] = label_counter;
            heap.push(S(values[i], i));
        }
    }

    // Flood fill from minima in order of increasing value
    while let Some(S(_, pi)) = heap.pop() {
        let label = labels[pi];
        if label == 0 {
            continue;
        }

        for ni in neighbors_of(pi) {
            if labels[ni] == 0 {
                labels[ni] = label;
                heap.push(S(values[ni], ni));
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WatershedLabel",
            labels_f,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_valleys() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        // Two valleys: minimum at index 1 and index 5
        let values = vec![3.0, 0.0, 3.0, 5.0, 3.0, 0.0, 3.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalars", values, 1)));

        let result = watershed_segment(&img, "scalars");
        let arr = result.point_data().get_array("WatershedLabel").unwrap();
        let mut buf = [0.0f64];

        // Voxel 1 and voxel 5 should have different labels
        arr.tuple_as_f64(1, &mut buf);
        let label_a: f64 = buf[0];
        arr.tuple_as_f64(5, &mut buf);
        let label_b: f64 = buf[0];

        assert!(label_a > 0.0);
        assert!(label_b > 0.0);
        assert!(
            (label_a - label_b).abs() > 0.5,
            "two minima should get different labels"
        );
    }

    #[test]
    fn all_voxels_labeled() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        let values = vec![2.0, 1.0, 0.0, 1.0, 2.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", values, 1)));

        let result = watershed_segment(&img, "s");
        let arr = result.point_data().get_array("WatershedLabel").unwrap();
        let mut buf = [0.0f64];
        for i in 0..5 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] > 0.0, "voxel {} should be labeled", i);
        }
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = watershed_segment(&img, "nonexistent");
        assert_eq!(result.dimensions(), [3, 3, 1]);
        assert!(result.point_data().get_array("WatershedLabel").is_none());
    }
}
