use crate::data::ImageData;
use std::collections::HashMap;

/// Statistics for each labeled region in a segmented ImageData.
#[derive(Debug, Clone, Default)]
pub struct LabelStat {
    pub label: i64,
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

/// Compute per-label statistics of a scalar field.
///
/// `labels_name` is the array with integer labels, `values_name` is the
/// scalar field to measure. Returns statistics for each unique label.
pub fn label_statistics(input: &ImageData, labels_name: &str, values_name: &str) -> Vec<LabelStat> {
    let labels_arr = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return vec![],
    };
    let values_arr = match input.point_data().get_array(values_name) {
        Some(a) => a,
        None => return vec![],
    };
    let n = labels_arr.num_tuples().min(values_arr.num_tuples());

    let mut bl = [0.0f64];
    let mut bv = [0.0f64];
    let mut stats: HashMap<i64, (usize, f64, f64, f64, f64)> = HashMap::new(); // (count,sum,sum2,min,max)

    for i in 0..n {
        labels_arr.tuple_as_f64(i, &mut bl);
        values_arr.tuple_as_f64(i, &mut bv);
        let label = bl[0] as i64;
        let v = bv[0];
        let e = stats
            .entry(label)
            .or_insert((0, 0.0, 0.0, f64::MAX, f64::MIN));
        e.0 += 1;
        e.1 += v;
        e.2 += v * v;
        e.3 = e.3.min(v);
        e.4 = e.4.max(v);
    }

    let mut result: Vec<LabelStat> = stats
        .into_iter()
        .map(|(label, (count, sum, _, min, max))| LabelStat {
            label,
            count,
            mean: sum / count as f64,
            min,
            max,
            sum,
        })
        .collect();
    result.sort_by_key(|s| s.label);
    result
}

/// Count voxels per label.
pub fn label_counts(input: &ImageData, labels_name: &str) -> Vec<(i64, usize)> {
    let arr = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return vec![],
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as i64).or_insert(0) += 1;
    }
    let mut result: Vec<(i64, usize)> = counts.into_iter().collect();
    result.sort_by_key(|&(l, _)| l);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn per_label_stats() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "labels",
                vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vals",
                vec![10.0, 20.0, 30.0, 100.0, 200.0, 300.0],
                1,
            )));

        let stats = label_statistics(&img, "labels", "vals");
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].label, 1);
        assert_eq!(stats[0].count, 3);
        assert!((stats[0].mean - 20.0).abs() < 1e-10);
        assert_eq!(stats[1].label, 2);
        assert!((stats[1].mean - 200.0).abs() < 1e-10);
    }

    #[test]
    fn label_count() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![0.0, 1.0, 1.0, 2.0, 2.0],
                1,
            )));

        let counts = label_counts(&img, "l");
        assert_eq!(counts.len(), 3);
        assert_eq!(counts[0], (0, 1));
        assert_eq!(counts[1], (1, 2));
    }

    #[test]
    fn missing_arrays() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert!(label_statistics(&img, "a", "b").is_empty());
        assert!(label_counts(&img, "a").is_empty());
    }
}
