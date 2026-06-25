use crate::data::ImageData;

/// Compute cosine similarity between two ImageData arrays.
///
/// cos_sim = (A·B) / (||A|| × ||B||), ranges from -1 to 1.
pub fn image_cosine_similarity(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    if aa.num_components() != ba.num_components() {
        return 0.0;
    }
    let n = aa.num_tuples().min(ba.num_tuples());
    if n == 0 {
        return 0.0;
    }

    let num_components = aa.num_components();
    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut dot = 0.0;
    let mut na2 = 0.0;
    let mut nb2 = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            dot += buf_a[c] * buf_b[c];
            na2 += buf_a[c] * buf_a[c];
            nb2 += buf_b[c] * buf_b[c];
        }
    }

    let denom = (na2 * nb2).sqrt();
    if denom > 1e-15 {
        dot / denom
    } else {
        0.0
    }
}

/// Compute Euclidean distance between two ImageData arrays treated as vectors.
pub fn image_euclidean_distance(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    if aa.num_components() != ba.num_components() {
        return 0.0;
    }
    let n = aa.num_tuples().min(ba.num_tuples());
    if n == 0 {
        return 0.0;
    }

    let num_components = aa.num_components();
    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut sum = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            sum += (buf_a[c] - buf_b[c]).powi(2);
        }
    }
    sum.sqrt()
}

/// Compute L1 (Manhattan) distance between two ImageData arrays.
pub fn image_l1_distance(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    if aa.num_components() != ba.num_components() {
        return 0.0;
    }
    let n = aa.num_tuples().min(ba.num_tuples());
    if n == 0 {
        return 0.0;
    }

    let num_components = aa.num_components();
    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut sum = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            sum += (buf_a[c] - buf_b[c]).abs();
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn identical_cosine_1() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));

        let sim = image_cosine_similarity(&img, &img, "v");
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn orthogonal_cosine_0() {
        let mut a = ImageData::with_dimensions(4, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.0, 0.0],
                1,
            )));
        let mut b = ImageData::with_dimensions(4, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 0.0, 0.0],
                1,
            )));

        let sim = image_cosine_similarity(&a, &b, "v");
        assert!(sim.abs() < 1e-10);
    }

    #[test]
    fn euclidean_basic() {
        let mut a = ImageData::with_dimensions(2, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0],
                1,
            )));
        let mut b = ImageData::with_dimensions(2, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0],
                1,
            )));

        assert!((image_euclidean_distance(&a, &b, "v") - 5.0).abs() < 1e-10);
    }

    #[test]
    fn l1_basic() {
        let mut a = ImageData::with_dimensions(3, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 3], 1)));
        let mut b = ImageData::with_dimensions(3, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        assert!((image_l1_distance(&a, &b, "v") - 6.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        assert_eq!(image_cosine_similarity(&img, &img, "nope"), 0.0);
    }
}
