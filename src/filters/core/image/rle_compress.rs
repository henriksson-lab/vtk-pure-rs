use crate::data::ImageData;

/// Run-length encode a 1D ImageData scalar field.
///
/// Returns (value, count) pairs. Useful for sparse binary volumes.
pub fn image_rle_encode(input: &ImageData, scalars: &str) -> Vec<(f64, usize)> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };
    let n = arr.num_tuples();
    if n == 0 {
        return vec![];
    }

    let mut buf = [0.0f64];
    let mut runs = Vec::new();
    arr.tuple_as_f64(0, &mut buf);
    let mut cur_val = buf[0];
    let mut cur_count = 1;

    for i in 1..n {
        arr.tuple_as_f64(i, &mut buf);
        if (buf[0] - cur_val).abs() < 1e-15 {
            cur_count += 1;
        } else {
            runs.push((cur_val, cur_count));
            cur_val = buf[0];
            cur_count = 1;
        }
    }
    runs.push((cur_val, cur_count));
    runs
}

/// Compute the compression ratio of RLE encoding.
pub fn rle_compression_ratio(input: &ImageData, scalars: &str) -> f64 {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return 1.0,
    };
    let n = arr.num_tuples();
    if n == 0 {
        return 1.0;
    }
    let runs = image_rle_encode(input, scalars);
    runs.len() as f64 / n as f64 // lower = better compression
}

/// Count unique values in an ImageData scalar field.
pub fn count_unique_values(input: &ImageData, scalars: &str) -> usize {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return 0,
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut unique = std::collections::HashSet::new();
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        unique.insert((buf[0] * 1e10).round() as i64);
    }
    unique.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn rle_basic() {
        let mut img = ImageData::with_dimensions(8, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0],
                1,
            )));

        let runs = image_rle_encode(&img, "v");
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0], (1.0, 3));
        assert_eq!(runs[1], (2.0, 2));
        assert_eq!(runs[2], (3.0, 3));
    }

    #[test]
    fn compression_ratio() {
        let mut img = ImageData::with_dimensions(10, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0; 10],
                1,
            )));

        let ratio = rle_compression_ratio(&img, "v");
        assert!((ratio - 0.1).abs() < 1e-10); // 1 run / 10 values
    }

    #[test]
    fn unique_values() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 1.0, 3.0, 2.0],
                1,
            )));

        assert_eq!(count_unique_values(&img, "v"), 3);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert!(image_rle_encode(&img, "nope").is_empty());
    }
}
