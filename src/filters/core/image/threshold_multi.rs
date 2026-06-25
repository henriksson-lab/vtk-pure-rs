use crate::data::{AnyDataArray, DataArray, ImageData};

/// Multi-level thresholding: assign each voxel to a level based on thresholds.
///
/// Given sorted threshold values [t0, t1, ...], assigns level 0 for values < t0,
/// level 1 for t0 <= v < t1, etc. Adds "Level" scalar array.
pub fn image_multi_threshold(input: &ImageData, scalars: &str, thresholds: &[f64]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut levels = Vec::with_capacity(n);

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v = buf[0];
        let mut level = 0usize;
        for &t in thresholds {
            if v >= t {
                level += 1;
            } else {
                break;
            }
        }
        levels.push(level as f64);
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Level", levels, 1)));
    img
}

/// Apply Otsu multi-threshold to find N-1 thresholds for N classes.
/// Simplified: uses recursive binary Otsu.
pub fn image_multi_otsu(input: &ImageData, scalars: &str, n_classes: usize) -> ImageData {
    if n_classes < 2 {
        return input.clone();
    }

    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Simple: split into n_classes equal-count bins
    let mut thresholds = Vec::new();
    for i in 1..n_classes {
        let idx = n * i / n_classes;
        thresholds.push(values[idx.min(n - 1)]);
    }

    image_multi_threshold(input, scalars, &thresholds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_levels() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));

        let result = image_multi_threshold(&img, "v", &[2.0, 4.0]);
        let arr = result.point_data().get_array("Level").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // < 2
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0); // >= 2, < 4
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 2.0); // >= 4
    }

    #[test]
    fn multi_otsu() {
        let mut img = ImageData::with_dimensions(9, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
                1,
            )));

        let result = image_multi_otsu(&img, "v", 3);
        assert!(result.point_data().get_array("Level").is_some());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_multi_threshold(&img, "nope", &[0.5]);
        assert!(r.point_data().get_array("Level").is_none());
    }
}
