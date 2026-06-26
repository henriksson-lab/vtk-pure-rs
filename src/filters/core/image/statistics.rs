use crate::data::ImageData;

/// Compute statistics of an ImageData scalar field.
#[derive(Debug, Clone)]
pub struct ImageStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub sum: f64,
    pub count: usize,
    pub num_nonzero: usize,
}

/// Compute statistics of a scalar array in an ImageData.
pub fn image_statistics(input: &ImageData, scalars: &str) -> Option<ImageStats> {
    let arr = input.point_data().get_array(scalars)?;
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut nonzero = 0;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v = buf[0];
        min_v = min_v.min(v);
        max_v = max_v.max(v);
        sum += v;
        sum_sq += v * v;
        if v.abs() > 1e-15 {
            nonzero += 1;
        }
    }

    let mean = sum / n as f64;
    let variance = if n > 1 {
        (sum_sq - sum * mean) / (n - 1) as f64
    } else {
        0.0
    };

    Some(ImageStats {
        min: min_v,
        max: max_v,
        mean,
        variance: variance.max(0.0),
        std_dev: variance.max(0.0).sqrt(),
        sum,
        count: n,
        num_nonzero: nonzero,
    })
}

/// Compute a histogram of an ImageData scalar field.
///
/// Returns (bin_centers, counts) with `n_bins` bins spanning [min, max].
pub fn image_histogram(
    input: &ImageData,
    scalars: &str,
    n_bins: usize,
) -> Option<(Vec<f64>, Vec<usize>)> {
    let arr = input.point_data().get_array(scalars)?;
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }
    let n_bins = n_bins.max(1);

    let mut buf = [0.0f64];
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }

    let bw = ((max_v - min_v) / n_bins as f64).max(1e-15);

    let centers: Vec<f64> = (0..n_bins).map(|i| min_v + (i as f64 + 0.5) * bw).collect();
    let mut counts = vec![0usize; n_bins];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = if buf[0] == max_v {
            n_bins - 1
        } else {
            ((buf[0] - min_v) / bw).floor().max(0.0) as usize
        };
        counts[bin.min(n_bins - 1)] += 1;
    }

    Some((centers, counts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn make_img() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = (1..=9).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn basic_stats() {
        let img = make_img();
        let stats = image_statistics(&img, "val").unwrap();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 9.0);
        assert_eq!(stats.mean, 5.0);
        assert_eq!(stats.sum, 45.0);
        assert_eq!(stats.count, 9);
        assert_eq!(stats.num_nonzero, 9);
    }

    #[test]
    fn histogram() {
        let img = make_img();
        let (centers, counts) = image_histogram(&img, "val", 4).unwrap();
        assert_eq!(centers.len(), 4);
        assert_eq!(counts.len(), 4);
        let total: usize = counts.iter().sum();
        assert_eq!(total, 9);
    }

    #[test]
    fn missing_array() {
        let img = make_img();
        assert!(image_statistics(&img, "nope").is_none());
        assert!(image_histogram(&img, "nope", 5).is_none());
    }
}
