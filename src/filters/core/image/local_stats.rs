//! Local statistics on ImageData: per-voxel neighborhood mean, variance, range.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local mean within a cubic neighborhood of given radius.
pub fn local_mean(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    local_stat(image, array_name, radius, "LocalMean", |vals| {
        vals.iter().sum::<f64>() / vals.len() as f64
    })
}

/// Compute local variance within a cubic neighborhood.
pub fn local_variance(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    local_stat(image, array_name, radius, "LocalVariance", |vals| {
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64
    })
}

/// Compute local range (max - min) within a cubic neighborhood.
pub fn local_range(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    local_stat(image, array_name, radius, "LocalRange", |vals| {
        vals.iter().cloned().fold(f64::MIN, f64::max)
            - vals.iter().cloned().fold(f64::MAX, f64::min)
    })
}

/// Compute local entropy within a neighborhood (using histogram).
pub fn local_entropy(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    local_stat(image, array_name, radius, "LocalEntropy", |vals| {
        let n = vals.len();
        if n == 0 {
            return 0.0;
        }
        // Build histogram with sqrt(n) bins
        let nbins = (n as f64).sqrt().ceil() as usize;
        let min = vals.iter().cloned().fold(f64::MAX, f64::min);
        let max = vals.iter().cloned().fold(f64::MIN, f64::max);
        let range = (max - min).max(1e-15);
        let mut counts = vec![0usize; nbins];
        for &v in vals {
            let bin = ((v - min) / range * nbins as f64) as usize;
            counts[bin.min(nbins - 1)] += 1;
        }
        let mut entropy = 0.0;
        for &c in &counts {
            let p = c as f64 / n as f64;
            if p > 1e-15 {
                entropy -= p * p.ln();
            }
        }
        entropy
    })
}

fn local_stat(
    image: &ImageData,
    array_name: &str,
    radius: usize,
    result_name: &str,
    f: impl Fn(&[f64]) -> f64,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let r = radius as i64;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut output = Vec::with_capacity(n);

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let mut nvals = Vec::new();
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let nx = ix as i64 + dx;
                            let ny = iy as i64 + dy;
                            let nz = iz as i64 + dz;
                            if nx >= 0
                                && ny >= 0
                                && nz >= 0
                                && (nx as usize) < dims[0]
                                && (ny as usize) < dims[1]
                                && (nz as usize) < dims[2]
                            {
                                nvals.push(
                                    vals[nx as usize
                                        + ny as usize * dims[0]
                                        + nz as usize * dims[0] * dims[1]],
                                );
                            }
                        }
                    }
                }
                output.push(f(&nvals));
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            result_name,
            output,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mean() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = local_mean(&img, "v", 1);
        assert!(result.point_data().get_array("LocalMean").is_some());
    }
    #[test]
    fn variance() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let result = local_variance(&img, "v", 1);
        assert!(result.point_data().get_array("LocalVariance").is_some());
    }
    #[test]
    fn range() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = local_range(&img, "v", 1);
        let arr = result.point_data().get_array("LocalRange").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2 + 2 * 5, &mut buf);
        assert!(buf[0] > 0.0); // non-constant neighborhood
    }
    #[test]
    fn entropy() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let result = local_entropy(&img, "v", 1);
        assert!(result.point_data().get_array("LocalEntropy").is_some());
    }
}
