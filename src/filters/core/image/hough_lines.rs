//! Hough transform for line detection in images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Detected line in Hough space.
pub struct HoughLine {
    pub rho: f64,
    pub theta: f64,
    pub votes: usize,
}

/// Compute Hough accumulator for line detection.
pub fn hough_accumulator(
    input: &ImageData,
    scalars: &str,
    theta_bins: usize,
    rho_bins: usize,
) -> ImageData {
    if theta_bins == 0 || rho_bins == 0 {
        return input.clone();
    }
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    if nx == 0 || ny == 0 {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let diag = ((nx * nx + ny * ny) as f64).sqrt();
    let mut acc = vec![0.0f64; theta_bins * rho_bins];

    for iy in 0..ny {
        for ix in 0..nx {
            arr.tuple_as_f64(ix + iy * nx, &mut buf);
            if buf[0] < 0.5 {
                continue;
            }
            for ti in 0..theta_bins {
                let theta = std::f64::consts::PI * ti as f64 / theta_bins as f64;
                let rho = ix as f64 * theta.cos() + iy as f64 * theta.sin();
                let ri = ((rho + diag) / (2.0 * diag) * rho_bins as f64) as usize;
                if ri < rho_bins {
                    acc[ti + ri * theta_bins] += 1.0;
                }
            }
        }
    }

    ImageData::with_dimensions(theta_bins, rho_bins, 1)
        .with_spacing([
            std::f64::consts::PI / theta_bins as f64,
            2.0 * diag / rho_bins as f64,
            1.0,
        ])
        .with_origin([0.0, -diag, 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Accumulator",
            acc,
            1,
        )))
}

/// Detect top N lines from Hough accumulator.
pub fn detect_lines(
    input: &ImageData,
    scalars: &str,
    theta_bins: usize,
    rho_bins: usize,
    n: usize,
) -> Vec<HoughLine> {
    if theta_bins == 0 || rho_bins == 0 || n == 0 {
        return Vec::new();
    }
    let acc_img = hough_accumulator(input, scalars, theta_bins, rho_bins);
    let arr = match acc_img.point_data().get_array("Accumulator") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let dims = acc_img.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let diag = {
        let d = input.dimensions();
        ((d[0] * d[0] + d[1] * d[1]) as f64).sqrt()
    };

    let mut buf = [0.0f64];
    let mut peaks: Vec<(usize, f64)> = (0..nx * ny)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (i, buf[0])
        })
        .collect();
    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    peaks
        .iter()
        .take(n)
        .map(|&(idx, votes)| {
            let ti = idx % theta_bins;
            let ri = idx / theta_bins;
            HoughLine {
                theta: std::f64::consts::PI * ti as f64 / theta_bins as f64,
                rho: -diag + 2.0 * diag * ri as f64 / rho_bins as f64,
                votes: votes as usize,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_accumulator() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (y - 10.0).abs() < 0.5 {
                    1.0
                } else {
                    0.0
                } // horizontal line
            },
        );
        let acc = hough_accumulator(&img, "v", 180, 100);
        assert_eq!(acc.dimensions(), [180, 100, 1]);
    }
    #[test]
    fn test_detect() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (y - 10.0).abs() < 0.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let lines = detect_lines(&img, "v", 90, 50, 3);
        assert!(!lines.is_empty());
        assert!(lines[0].votes > 0);
    }
}
