//! Region of interest (ROI) analysis on ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData, Table};

/// Extract statistics within a spherical ROI.
pub fn sphere_roi_stats(
    image: &ImageData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
) -> Option<(f64, f64, f64, f64, usize)> {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return None,
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let r2 = radius * radius;
    let mut buf = [0.0f64];
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let mut count = 0;

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let x = org[0] + ix as f64 * sp[0];
                let y = org[1] + iy as f64 * sp[1];
                let z = org[2] + iz as f64 * sp[2];
                if (x - center[0]).powi(2) + (y - center[1]).powi(2) + (z - center[2]).powi(2) > r2
                {
                    continue;
                }
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    sum += buf[0];
                    sum2 += buf[0] * buf[0];
                    min_v = min_v.min(buf[0]);
                    max_v = max_v.max(buf[0]);
                    count += 1;
                }
            }
        }
    }

    if count > 0 {
        let mean = sum / count as f64;
        let std = ((sum2 / count as f64) - mean * mean).max(0.0).sqrt();
        Some((min_v, max_v, mean, std, count))
    } else {
        None
    }
}

/// Extract statistics within a box ROI.
pub fn box_roi_stats(
    image: &ImageData,
    array_name: &str,
    min: [f64; 3],
    max: [f64; 3],
) -> Option<(f64, f64, f64, f64, usize)> {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return None,
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let mut buf = [0.0f64];
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let mut count = 0;

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let x = org[0] + ix as f64 * sp[0];
                let y = org[1] + iy as f64 * sp[1];
                let z = org[2] + iz as f64 * sp[2];
                if x < min[0] || x > max[0] || y < min[1] || y > max[1] || z < min[2] || z > max[2]
                {
                    continue;
                }
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    sum += buf[0];
                    sum2 += buf[0] * buf[0];
                    min_v = min_v.min(buf[0]);
                    max_v = max_v.max(buf[0]);
                    count += 1;
                }
            }
        }
    }

    if count > 0 {
        let mean = sum / count as f64;
        let std = ((sum2 / count as f64) - mean * mean).max(0.0).sqrt();
        Some((min_v, max_v, mean, std, count))
    } else {
        None
    }
}

/// Create a mask from an ROI (sphere or box) and apply to an array.
pub fn mask_sphere_roi(image: &ImageData, center: [f64; 3], radius: f64) -> ImageData {
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let r2 = radius * radius;
    let n = dims[0] * dims[1] * dims[2];
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (dims[0] * dims[1]);
            let rem = idx % (dims[0] * dims[1]);
            let iy = rem / dims[0];
            let ix = rem % dims[0];
            let x = org[0] + ix as f64 * sp[0];
            let y = org[1] + iy as f64 * sp[1];
            let z = org[2] + iz as f64 * sp[2];
            if (x - center[0]).powi(2) + (y - center[1]).powi(2) + (z - center[2]).powi(2) <= r2 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ROIMask", data, 1)));
    result
}

/// Multi-ROI analysis: compute stats for multiple spherical ROIs.
pub fn multi_roi_analysis(image: &ImageData, array_name: &str, rois: &[([f64; 3], f64)]) -> Table {
    let mut roi_id = Vec::new();
    let mut means = Vec::new();
    let mut stds = Vec::new();
    let mut counts = Vec::new();
    for (i, &(center, radius)) in rois.iter().enumerate() {
        if let Some((_, _, mean, std, count)) = sphere_roi_stats(image, array_name, center, radius)
        {
            roi_id.push(i as f64);
            means.push(mean);
            stds.push(std);
            counts.push(count as f64);
        }
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec("ROI", roi_id, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Mean", means, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("StdDev", stds, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "VoxelCount",
            counts,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_stats() {
        let img = ImageData::from_function(
            [10, 10, 10],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let stats = sphere_roi_stats(&img, "v", [5.0, 5.0, 5.0], 3.0).unwrap();
        assert!(stats.4 > 10); // count
        assert!(stats.2 > 3.0 && stats.2 < 7.0); // mean near center X
    }
    #[test]
    fn box_stats() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let stats = box_roi_stats(&img, "v", [2.0, 2.0, -1.0], [8.0, 8.0, 1.0]).unwrap();
        assert!(stats.4 > 10);
    }
    #[test]
    fn mask() {
        let img = ImageData::with_dimensions(10, 10, 10).with_spacing([1.0, 1.0, 1.0]);
        let result = mask_sphere_roi(&img, [5.0, 5.0, 5.0], 3.0);
        assert!(result.point_data().get_array("ROIMask").is_some());
    }
    #[test]
    fn multi_roi() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let table =
            multi_roi_analysis(&img, "v", &[([3.0, 3.0, 0.0], 2.0), ([7.0, 7.0, 0.0], 2.0)]);
        assert_eq!(table.num_rows(), 2);
    }
}
