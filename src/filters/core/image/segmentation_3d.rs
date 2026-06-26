//! 3D image segmentation: region growing, watershed seeds, label analysis.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D connected threshold region growing.
///
/// Starting from seed voxels, grows regions where scalar values are
/// within [lower, upper]. Adds a "RegionMask" array.
pub fn region_grow_3d(
    image: &ImageData,
    array_name: &str,
    seeds: &[[usize; 3]],
    lower: f64,
    upper: f64,
) -> ImageData {
    let dims = image.dimensions();
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image.clone(),
    };
    let n = dims[0] * dims[1] * dims[2];
    if arr.num_tuples() < n {
        return image.clone();
    }
    let mut mask = vec![0.0f64; n];
    let mut buf = [0.0f64];

    let idx = |x: usize, y: usize, z: usize| x + y * dims[0] + z * dims[0] * dims[1];

    let mut queue = std::collections::VecDeque::new();
    for s in seeds {
        if s[0] < dims[0] && s[1] < dims[1] && s[2] < dims[2] {
            let i = idx(s[0], s[1], s[2]);
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= lower && buf[0] <= upper {
                mask[i] = 1.0;
                queue.push_back([s[0], s[1], s[2]]);
            }
        }
    }

    while let Some([x, y, z]) = queue.pop_front() {
        let neighbors: Vec<[usize; 3]> = [
            [x.wrapping_sub(1), y, z],
            [x + 1, y, z],
            [x, y.wrapping_sub(1), z],
            [x, y + 1, z],
            [x, y, z.wrapping_sub(1)],
            [x, y, z + 1],
        ]
        .into_iter()
        .filter(|[nx, ny, nz]| *nx < dims[0] && *ny < dims[1] && *nz < dims[2])
        .collect();

        for [nx, ny, nz] in neighbors {
            let ni = idx(nx, ny, nz);
            if mask[ni] > 0.5 {
                continue;
            }
            arr.tuple_as_f64(ni, &mut buf);
            if buf[0] >= lower && buf[0] <= upper {
                mask[ni] = 1.0;
                queue.push_back([nx, ny, nz]);
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionMask",
            mask,
            1,
        )));
    result
}

/// Count labeled regions in a 3D label map.
pub fn count_labels_3d(
    image: &ImageData,
    array_name: &str,
) -> std::collections::HashMap<i64, usize> {
    let mut counts = std::collections::HashMap::new();
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return counts,
    };
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as i64).or_insert(0) += 1;
    }
    counts
}

/// Compute per-label statistics: centroid, volume (voxel count), mean value.
pub fn label_statistics_3d(
    image: &ImageData,
    label_name: &str,
    value_name: &str,
) -> std::collections::HashMap<i64, (f64, [f64; 3], usize)> {
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let label_arr = match image.point_data().get_array(label_name) {
        Some(a) if a.num_tuples() >= n => a,
        None => return std::collections::HashMap::new(),
        _ => return std::collections::HashMap::new(),
    };
    let val_arr = image.point_data().get_array(value_name);
    if let Some(va) = val_arr {
        if va.num_tuples() < n {
            return std::collections::HashMap::new();
        }
    }

    let mut stats: std::collections::HashMap<i64, (f64, [f64; 3], usize)> =
        std::collections::HashMap::new();
    let mut lbuf = [0.0f64];
    let mut vbuf = [0.0f64];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                label_arr.tuple_as_f64(idx, &mut lbuf);
                let label = lbuf[0] as i64;
                if label <= 0 {
                    continue;
                }

                let val = if let Some(va) = val_arr {
                    va.tuple_as_f64(idx, &mut vbuf);
                    vbuf[0]
                } else {
                    1.0
                };

                let [x, y, z] = image.point_from_ijk(ix, iy, iz);

                let entry = stats.entry(label).or_insert((0.0, [0.0; 3], 0));
                entry.0 += val;
                entry.1[0] += x;
                entry.1[1] += y;
                entry.1[2] += z;
                entry.2 += 1;
            }
        }
    }

    // Normalize centroids and compute mean values
    for (_, (sum_val, centroid, count)) in &mut stats {
        if *count > 0 {
            centroid[0] /= *count as f64;
            centroid[1] /= *count as f64;
            centroid[2] /= *count as f64;
            *sum_val /= *count as f64; // mean value
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_grow() {
        let image = ImageData::from_function(
            [10, 10, 10],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| {
                if (x - 5.0).powi(2) + (y - 5.0).powi(2) + (z - 5.0).powi(2) < 9.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = region_grow_3d(&image, "val", &[[5, 5, 5]], 0.5, 1.5);
        let mask = result.point_data().get_array("RegionMask").unwrap();
        let mut buf = [0.0f64];
        mask.tuple_as_f64(5 + 5 * 10 + 5 * 100, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn count_labels() {
        let mut image = ImageData::with_dimensions(5, 5, 1);
        let labels = vec![
            0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 2.0, 2.0, 1.0, 1.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("L", labels, 1)));
        let counts = count_labels_3d(&image, "L");
        assert!(counts.contains_key(&1));
        assert!(counts.contains_key(&2));
    }

    #[test]
    fn label_stats() {
        let mut image = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x,
        );
        let labels: Vec<f64> = (0..25).map(|i| if i < 12 { 1.0 } else { 2.0 }).collect();
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("L", labels, 1)));
        let stats = label_statistics_3d(&image, "L", "val");
        assert!(stats.contains_key(&1));
        assert!(stats.contains_key(&2));
    }

    #[test]
    fn label_stats_use_image_extent_for_centroids() {
        let mut image = ImageData::with_dimensions(2, 1, 1);
        image.set_extent([10, 11, 0, 0, 0, 0]);
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "L",
                vec![1.0, 1.0],
                1,
            )));
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![2.0, 4.0],
                1,
            )));

        let stats = label_statistics_3d(&image, "L", "val");
        let (mean, centroid, count) = stats.get(&1).unwrap();
        assert_eq!(*mean, 3.0);
        assert_eq!(*count, 2);
        assert_eq!(*centroid, [10.5, 0.0, 0.0]);
    }
}
