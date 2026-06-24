//! Volume data analysis filters for ImageData.
//!
//! Statistical analysis, feature detection, and region analysis
//! specifically designed for 3D volumetric data.

use crate::data::ImageData;

/// Compute volume-weighted histogram of a scalar field.
pub fn volume_histogram(image: &ImageData, array_name: &str, n_bins: usize) -> Vec<(f64, usize)> {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Vec::new(),
    };

    let n = arr.num_tuples();
    if n == 0 || n_bins == 0 {
        return Vec::new();
    }

    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    if (max_v - min_v).abs() < 1e-15 {
        max_v = min_v + 1.0;
    }

    let bin_width = (max_v - min_v) / n_bins as f64;
    let mut counts = vec![0usize; n_bins];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let bin = ((buf[0] - min_v) / bin_width) as usize;
        counts[bin.min(n_bins - 1)] += 1;
    }

    (0..n_bins)
        .map(|i| (min_v + (i as f64 + 0.5) * bin_width, counts[i]))
        .collect()
}

/// Compute the volume (number of voxels × voxel volume) above a threshold.
pub fn volume_above_threshold(image: &ImageData, array_name: &str, threshold: f64) -> f64 {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };
    let spacing = image.spacing();
    let voxel_vol = spacing[0] * spacing[1] * spacing[2];

    let mut count = 0usize;
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= threshold {
            count += 1;
        }
    }
    count as f64 * voxel_vol
}

/// Compute the centroid of voxels above a threshold (center of mass).
pub fn volume_centroid(image: &ImageData, array_name: &str, threshold: f64) -> Option<[f64; 3]> {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return None,
    };
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    let mut total = 0.0;
    let mut buf = [0.0f64];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx >= arr.num_tuples() {
                    continue;
                }
                arr.tuple_as_f64(idx, &mut buf);
                if buf[0] >= threshold {
                    let w = buf[0];
                    cx += w * (origin[0] + ix as f64 * spacing[0]);
                    cy += w * (origin[1] + iy as f64 * spacing[1]);
                    cz += w * (origin[2] + iz as f64 * spacing[2]);
                    total += w;
                }
            }
        }
    }

    if total > 1e-15 {
        Some([cx / total, cy / total, cz / total])
    } else {
        None
    }
}

/// Compute scalar field integral over the volume.
pub fn volume_integral(image: &ImageData, array_name: &str) -> f64 {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return 0.0,
    };
    let spacing = image.spacing();
    let voxel_vol = spacing[0] * spacing[1] * spacing[2];

    let mut sum = 0.0;
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        sum += buf[0];
    }
    sum * voxel_vol
}

/// Compute min, max, mean, std of a volume scalar field.
pub fn volume_statistics(image: &ImageData, array_name: &str) -> Option<(f64, f64, f64, f64)> {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return None,
    };
    let n = arr.num_tuples();
    if n == 0 {
        return None;
    }

    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let mut sum = 0.0;
    let mut sum2 = 0.0;

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v = buf[0];
        min_v = min_v.min(v);
        max_v = max_v.max(v);
        sum += v;
        sum2 += v * v;
    }

    let mean = sum / n as f64;
    let variance = sum2 / n as f64 - mean * mean;
    Some((min_v, max_v, mean, variance.max(0.0).sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image() -> ImageData {
        ImageData::from_function(
            [10, 10, 10],
            [0.1, 0.1, 0.1],
            [0.0, 0.0, 0.0],
            "density",
            |x, y, z| (x * x + y * y + z * z).sqrt(),
        )
    }

    #[test]
    fn histogram() {
        let hist = volume_histogram(&make_image(), "density", 10);
        assert_eq!(hist.len(), 10);
        let total: usize = hist.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn volume_above() {
        let vol = volume_above_threshold(&make_image(), "density", 0.5);
        assert!(vol > 0.0);
    }

    #[test]
    fn centroid() {
        let c = volume_centroid(&make_image(), "density", 0.0).unwrap();
        // Centroid should be near the center-ish (weighted by distance)
        assert!(c[0] > 0.0 && c[0] < 1.0);
    }

    #[test]
    fn integral() {
        let val = volume_integral(&make_image(), "density");
        assert!(val > 0.0);
    }

    #[test]
    fn stats() {
        let (min, max, mean, std) = volume_statistics(&make_image(), "density").unwrap();
        assert!(min >= 0.0);
        assert!(max > min);
        assert!(mean > 0.0);
        assert!(std >= 0.0);
    }
}
