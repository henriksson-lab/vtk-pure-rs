use crate::data::{AnyDataArray, DataArray, ImageData};

/// K-nearest-neighbor classification of an ImageData scalar field.
///
/// Given labeled training voxels (where `labels_name` > 0), classifies
/// unlabeled voxels based on majority vote of k nearest labeled neighbors.
pub fn image_knn_classify(
    input: &ImageData,
    values_name: &str,
    labels_name: &str,
    k: usize,
) -> ImageData {
    let va = match input.point_data().get_array(values_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let la = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = va.num_tuples().min(la.num_tuples());
    let k = k.max(1);

    let mut vbuf = [0.0f64];
    let mut lbuf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            va.tuple_as_f64(i, &mut vbuf);
            vbuf[0]
        })
        .collect();
    let labels: Vec<i64> = (0..n)
        .map(|i| {
            la.tuple_as_f64(i, &mut lbuf);
            lbuf[0] as i64
        })
        .collect();

    // Collect training samples
    let training: Vec<(f64, i64)> = values
        .iter()
        .zip(labels.iter())
        .filter(|(_, &l)| l > 0)
        .map(|(&v, &l)| (v, l))
        .collect();

    if training.is_empty() {
        return input.clone();
    }

    let mut classified = vec![0.0f64; n];
    for i in 0..n {
        if labels[i] > 0 {
            classified[i] = labels[i] as f64;
            continue;
        }

        // Find k nearest training samples by value distance
        let mut dists: Vec<(f64, i64)> = training
            .iter()
            .map(|&(tv, tl)| ((values[i] - tv).abs(), tl))
            .collect();
        dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Majority vote
        let mut votes = std::collections::HashMap::new();
        for &(_, label) in dists.iter().take(k) {
            *votes.entry(label).or_insert(0usize) += 1;
        }
        classified[i] = votes
            .into_iter()
            .max_by_key(|&(_, c)| c)
            .map(|(l, _)| l as f64)
            .unwrap_or(0.0);
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Classified",
            classified,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_basic() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 5.0, 9.0, 10.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![1.0, 0.0, 0.0, 0.0, 2.0],
                1,
            )));

        let result = image_knn_classify(&img, "v", "l", 1);
        let arr = result.point_data().get_array("Classified").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // near label 1
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 2.0); // near label 2
    }

    #[test]
    fn labeled_preserved() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 3], 1)));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![5.0, 0.0, 3.0],
                1,
            )));

        let result = image_knn_classify(&img, "v", "l", 1);
        let arr = result.point_data().get_array("Classified").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0); // preserved
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 3.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_knn_classify(&img, "nope", "nope2", 3);
        assert!(r.point_data().get_array("Classified").is_none());
    }
}
