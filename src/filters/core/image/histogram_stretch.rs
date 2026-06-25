//! Histogram stretching and contrast enhancement.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Linear contrast stretch to [out_min, out_max].
pub fn linear_stretch(input: &ImageData, scalars: &str, out_min: f64, out_max: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| out_min + (v - mn) / range * (out_max - out_min))
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Percentile contrast stretch (clip tails).
pub fn percentile_stretch(
    input: &ImageData,
    scalars: &str,
    low_pct: f64,
    high_pct: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo = sorted[((n as f64 * low_pct) as usize).min(n - 1)];
    let hi = sorted[((n as f64 * high_pct) as usize).min(n - 1)];
    let range = if (hi - lo).abs() < 1e-15 {
        1.0
    } else {
        hi - lo
    };
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| ((v - lo) / range).clamp(0.0, 1.0))
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// CLAHE-like local contrast enhancement (simplified).
pub fn local_contrast_enhance(input: &ImageData, scalars: &str, tile_size: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    if n == 0 || nx == 0 || ny == 0 {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let ts = tile_size.max(2);
    let half = ts / 2;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = (idx / nx) % ny;
            let ix = idx % nx;
            let x0 = if ix >= half { ix - half } else { 0 };
            let x1 = (ix + half).min(nx - 1);
            let y0 = if iy >= half { iy - half } else { 0 };
            let y1 = (iy + half).min(ny - 1);
            let mut mn = f64::INFINITY;
            let mut mx = f64::NEG_INFINITY;
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    let v = vals[xx + yy * nx];
                    mn = mn.min(v);
                    mx = mx.max(v);
                }
            }
            let range = if (mx - mn).abs() < 1e-15 {
                1.0
            } else {
                mx - mn
            };
            (vals[idx] - mn) / range
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_linear() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 10.0,
        );
        let r = linear_stretch(&img, "v", 0.0, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
    }
    #[test]
    fn test_percentile() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = percentile_stretch(&img, "v", 0.1, 0.9);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn test_local_contrast() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = local_contrast_enhance(&img, "v", 4);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
}
