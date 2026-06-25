use crate::data::{AnyDataArray, DataArray, ImageData};
use std::collections::VecDeque;

/// Connected threshold segmentation: region grow from seeds where value is in range.
///
/// Starting from `seeds` (index positions), grows to 6-connected neighbors
/// where the scalar is in [lower, upper]. Adds "Segmented" binary array.
pub fn connected_threshold(
    input: &ImageData,
    scalars: &str,
    seeds: &[usize],
    lower: f64,
    upper: f64,
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
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut segmented = vec![0.0f64; n];
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    for &s in seeds {
        if s < n && values[s] >= lower && values[s] <= upper {
            segmented[s] = 1.0;
            visited[s] = true;
            queue.push_back(s);
        }
    }

    while let Some(pi) = queue.pop_front() {
        let i = pi % nx;
        let j = (pi / nx) % ny;
        let k = pi / (ny * nx);
        let nbrs = [
            (i.wrapping_sub(1), j, k),
            (i + 1, j, k),
            (i, j.wrapping_sub(1), k),
            (i, j + 1, k),
            (i, j, k.wrapping_sub(1)),
            (i, j, k + 1),
        ];
        for &(ni, nj, nk) in &nbrs {
            if ni < nx && nj < ny && nk < nz {
                let nidx = idx(ni, nj, nk);
                if !visited[nidx] && values[nidx] >= lower && values[nidx] <= upper {
                    visited[nidx] = true;
                    segmented[nidx] = 1.0;
                    queue.push_back(nidx);
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Segmented",
            segmented,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grow_region() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 10.0, 10.0, 0.0, 10.0, 10.0, 10.0],
                1,
            )));

        let result = connected_threshold(&img, "v", &[0], 5.0, 15.0);
        let arr = result.point_data().get_array("Segmented").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0); // barrier
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // unreachable
    }

    #[test]
    fn seed_outside_range() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = connected_threshold(&img, "v", &[0], 5.0, 10.0);
        let arr = result.point_data().get_array("Segmented").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn all_in_range() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = connected_threshold(&img, "v", &[4], 0.0, 10.0);
        let arr = result.point_data().get_array("Segmented").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = connected_threshold(&img, "nope", &[0], 0.0, 1.0);
        assert!(r.point_data().get_array("Segmented").is_none());
    }
}
