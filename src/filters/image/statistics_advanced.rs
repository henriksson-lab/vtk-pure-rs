//! Advanced image statistics: mutual information, joint histogram, entropy.

use crate::data::{AnyDataArray, DataArray, ImageData, Table};

/// Compute mutual information between two scalar arrays on the same ImageData.
pub fn mutual_information(image: &ImageData, array_a: &str, array_b: &str, n_bins: usize) -> f64 {
    let a = match image.point_data().get_array(array_a) {
        Some(x) => x,
        None => return 0.0,
    };
    let b = match image.point_data().get_array(array_b) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = a.num_tuples().min(b.num_tuples());
    if n == 0 {
        return 0.0;
    }

    let mut ab = [0.0f64];
    let mut bb = [0.0f64];

    // Find ranges
    let (mut a_min, mut a_max) = (f64::MAX, f64::MIN);
    let (mut b_min, mut b_max) = (f64::MAX, f64::MIN);
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        a_min = a_min.min(ab[0]);
        a_max = a_max.max(ab[0]);
        b_min = b_min.min(bb[0]);
        b_max = b_max.max(bb[0]);
    }
    let ar = (a_max - a_min).max(1e-15);
    let br = (b_max - b_min).max(1e-15);

    // Joint histogram
    let mut joint = vec![vec![0usize; n_bins]; n_bins];
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        let ai = (((ab[0] - a_min) / ar * n_bins as f64) as usize).min(n_bins - 1);
        let bi = (((bb[0] - b_min) / br * n_bins as f64) as usize).min(n_bins - 1);
        joint[ai][bi] += 1;
    }

    // Marginals
    let mut pa = vec![0.0f64; n_bins];
    let mut pb = vec![0.0f64; n_bins];
    for i in 0..n_bins {
        for j in 0..n_bins {
            let p = joint[i][j] as f64 / n as f64;
            pa[i] += p;
            pb[j] += p;
        }
    }

    // MI = sum p(a,b) * log(p(a,b) / (p(a)*p(b)))
    let mut mi = 0.0;
    for i in 0..n_bins {
        for j in 0..n_bins {
            let pab = joint[i][j] as f64 / n as f64;
            if pab > 1e-15 && pa[i] > 1e-15 && pb[j] > 1e-15 {
                mi += pab * (pab / (pa[i] * pb[j])).ln();
            }
        }
    }
    mi
}

/// Compute Shannon entropy of a scalar array.
pub fn scalar_entropy(image: &ImageData, array_name: &str, n_bins: usize) -> f64 {
    let arr = match image.point_data().get_array(array_name) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = arr.num_tuples();
    if n == 0 {
        return 0.0;
    }
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let mut counts = vec![0usize; n_bins];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = (((buf[0] - min_v) / range * n_bins as f64) as usize).min(n_bins - 1);
        counts[bin] += 1;
    }
    let mut entropy = 0.0;
    for &c in &counts {
        let p = c as f64 / n as f64;
        if p > 1e-15 {
            entropy -= p * p.ln();
        }
    }
    entropy
}

/// Compute joint histogram as a Table.
pub fn joint_histogram(image: &ImageData, array_a: &str, array_b: &str, n_bins: usize) -> Table {
    let a = match image.point_data().get_array(array_a) {
        Some(x) => x,
        None => return Table::new(),
    };
    let b = match image.point_data().get_array(array_b) {
        Some(x) => x,
        None => return Table::new(),
    };
    let n = a.num_tuples().min(b.num_tuples());
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];

    let (mut a_min, mut a_max) = (f64::MAX, f64::MIN);
    let (mut b_min, mut b_max) = (f64::MAX, f64::MIN);
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        a_min = a_min.min(ab[0]);
        a_max = a_max.max(ab[0]);
        b_min = b_min.min(bb[0]);
        b_max = b_max.max(bb[0]);
    }
    let ar = (a_max - a_min).max(1e-15);
    let br = (b_max - b_min).max(1e-15);

    let mut joint = vec![vec![0usize; n_bins]; n_bins];
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        let ai = (((ab[0] - a_min) / ar * n_bins as f64) as usize).min(n_bins - 1);
        let bi = (((bb[0] - b_min) / br * n_bins as f64) as usize).min(n_bins - 1);
        joint[ai][bi] += 1;
    }

    let mut a_data = Vec::new();
    let mut b_data = Vec::new();
    let mut c_data = Vec::new();
    let bw_a = ar / n_bins as f64;
    let bw_b = br / n_bins as f64;
    for i in 0..n_bins {
        for j in 0..n_bins {
            a_data.push(a_min + (i as f64 + 0.5) * bw_a);
            b_data.push(b_min + (j as f64 + 0.5) * bw_b);
            c_data.push(joint[i][j] as f64);
        }
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(array_a, a_data, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(array_b, b_data, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Count", c_data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mi_identical() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        );
        // Duplicate array as "b"
        let a = img.point_data().get_array("a").unwrap();
        let mut vals = Vec::new();
        let mut buf = [0.0f64];
        for i in 0..a.num_tuples() {
            a.tuple_as_f64(i, &mut buf);
            vals.push(buf[0]);
        }
        let mut img2 = img.clone();
        img2.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("b", vals, 1)));
        let mi = mutual_information(&img2, "a", "b", 10);
        assert!(mi > 0.0, "identical arrays should have positive MI");
    }
    #[test]
    fn entropy_uniform() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let e = scalar_entropy(&img, "v", 10);
        assert!(e > 0.0);
    }
    #[test]
    fn joint_hist() {
        let mut img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "a",
            |x, _, _| x,
        );
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                (0..100).map(|i| i as f64).collect(),
                1,
            )));
        let jh = joint_histogram(&img, "a", "b", 5);
        assert_eq!(jh.num_rows(), 25);
    }
}
