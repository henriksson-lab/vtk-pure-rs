use crate::data::{AnyDataArray, DataArray, ImageData};

/// Simple marker-based watershed segmentation on ImageData.
///
/// Propagates labels from seed voxels outward in order of increasing
/// scalar value (like water filling basins). Seeds are voxels where
/// `markers_name` array is > 0.
pub fn image_watershed(input: &ImageData, scalars: &str, markers_name: &str) -> ImageData {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let markers = match input.point_data().get_array(markers_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut labels = vec![0i64; n];

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
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let mut heap = BinaryHeap::new();

    // Initialize from markers
    for i in 0..n {
        markers.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.0 {
            labels[i] = buf[0] as i64;
            heap.push(S(values[i], i));
        }
    }

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    while let Some(S(_, pi)) = heap.pop() {
        let i = pi % nx;
        let j = (pi / nx) % ny;
        let k = pi / (ny * nx);
        let label = labels[pi];
        if label == 0 {
            continue;
        }

        let neighbors = [
            if i > 0 { Some(idx(i - 1, j, k)) } else { None },
            if i + 1 < nx {
                Some(idx(i + 1, j, k))
            } else {
                None
            },
            if j > 0 { Some(idx(i, j - 1, k)) } else { None },
            if j + 1 < ny {
                Some(idx(i, j + 1, k))
            } else {
                None
            },
            if k > 0 { Some(idx(i, j, k - 1)) } else { None },
            if k + 1 < nz {
                Some(idx(i, j, k + 1))
            } else {
                None
            },
        ];

        for nb in &neighbors {
            if let Some(ni) = nb {
                if labels[*ni] == 0 {
                    labels[*ni] = label;
                    heap.push(S(values[*ni], *ni));
                }
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WatershedLabels",
            labels_f,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_basins() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        // Valley at center: [5, 3, 1, 0, 1, 3, 5]
        let values = vec![5.0, 3.0, 1.0, 0.0, 1.0, 3.0, 5.0];
        let markers = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("m", markers, 1)));

        let result = image_watershed(&img, "v", "m");
        let arr = result.point_data().get_array("WatershedLabels").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(6, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn all_labeled() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 5], 1)));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0, 0.0, 0.0, 0.0],
                1,
            )));

        let result = image_watershed(&img, "v", "m");
        let arr = result.point_data().get_array("WatershedLabels").unwrap();
        let mut buf = [0.0f64];
        for i in 0..5 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

    #[test]
    fn missing_arrays() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_watershed(&img, "nope", "nope2");
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
