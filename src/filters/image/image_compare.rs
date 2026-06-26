//! Image comparison metrics (PSNR, MSE, MAE, SSIM-like).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Mean Squared Error between two images.
pub fn mse(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let (va, vb) = read_pair(a, b, scalars);
    if va.is_empty() {
        return 0.0;
    }
    va.iter()
        .zip(vb.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        / va.len() as f64
}

/// Compute Mean Absolute Error.
pub fn mae(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let (va, vb) = read_pair(a, b, scalars);
    if va.is_empty() {
        return 0.0;
    }
    va.iter()
        .zip(vb.iter())
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
        / va.len() as f64
}

/// Compute Peak Signal-to-Noise Ratio.
pub fn psnr(a: &ImageData, b: &ImageData, scalars: &str, max_val: f64) -> f64 {
    let m = mse(a, b, scalars);
    if m < 1e-30 {
        return f64::INFINITY;
    }
    10.0 * (max_val * max_val / m).log10()
}

/// Compute normalized cross-correlation.
pub fn normalized_cross_correlation(a: &ImageData, b: &ImageData, scalars: &str) -> f64 {
    let (va, vb) = read_pair(a, b, scalars);
    if va.is_empty() {
        return 0.0;
    }
    let n = va.len() as f64;
    let ma = va.iter().sum::<f64>() / n;
    let mb = vb.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for i in 0..va.len() {
        let x = va[i] - ma;
        let y = vb[i] - mb;
        num += x * y;
        da += x * x;
        db += y * y;
    }
    let denom = (da * db).sqrt();
    if denom < 1e-30 {
        0.0
    } else {
        num / denom
    }
}

/// Compute difference image (a - b).
pub fn difference_image(a: &ImageData, b: &ImageData, scalars: &str) -> ImageData {
    let (va, vb) = read_pair(a, b, scalars);
    if va.len() != a.num_points() {
        return a.clone();
    }
    let data: Vec<f64> = va.iter().zip(vb.iter()).map(|(x, y)| x - y).collect();
    let dims = a.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(a.spacing())
        .with_origin(a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Difference",
            data,
            1,
        )))
}

fn read_pair(a: &ImageData, b: &ImageData, scalars: &str) -> (Vec<f64>, Vec<f64>) {
    if a.dimensions() != b.dimensions() || a.spacing() != b.spacing() || a.origin() != b.origin() {
        return (vec![], vec![]);
    }
    let aa = match a.point_data().get_array(scalars) {
        Some(x) if x.num_components() == 1 => x,
        _ => return (vec![], vec![]),
    };
    let bb = match b.point_data().get_array(scalars) {
        Some(x) if x.num_components() == 1 => x,
        _ => return (vec![], vec![]),
    };
    let n = a.num_points();
    if aa.num_tuples() != n || bb.num_tuples() != n {
        return (vec![], vec![]);
    }
    let mut buf_a = [0.0f64];
    let va: Vec<f64> = (0..n)
        .map(|i| {
            aa.tuple_as_f64(i, &mut buf_a);
            buf_a[0]
        })
        .collect();
    let mut buf_b = [0.0f64];
    let vb: Vec<f64> = (0..n)
        .map(|i| {
            bb.tuple_as_f64(i, &mut buf_b);
            buf_b[0]
        })
        .collect();
    (va, vb)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_identical() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        assert!((mse(&img, &img, "v")).abs() < 1e-15);
        assert!(psnr(&img, &img, "v", 255.0).is_infinite());
        assert!((normalized_cross_correlation(&img, &img, "v") - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_different() {
        let a = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let b = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 10.0,
        );
        assert!((mse(&a, &b, "v") - 100.0).abs() < 1e-10);
        assert!((mae(&a, &b, "v") - 10.0).abs() < 1e-10);
    }

    #[test]
    fn mismatched_geometry_is_not_compared_as_prefix() {
        let a = ImageData::from_function(
            [2, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let b = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 2.0,
        );
        assert_eq!(mse(&a, &b, "v"), 0.0);
        assert_eq!(mae(&a, &b, "v"), 0.0);
        assert_eq!(difference_image(&a, &b, "v").dimensions(), a.dimensions());
    }
}
