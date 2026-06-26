//! Connected component analysis on binary ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData, Table};

/// Label connected components in a 3D binary image.
///
/// Uses 6-connectivity flood fill. Returns labeled image + component count.
pub fn label_components_3d(image: &ImageData, array_name: &str) -> (ImageData, usize) {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return (image.clone(), 0),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= 0.5
        })
        .collect();
    let mut labels = vec![0i64; n];
    let mut next_label = 1i64;

    let idx = |x: usize, y: usize, z: usize| x + y * dims[0] + z * dims[0] * dims[1];

    for start in 0..n {
        if !fg[start] || labels[start] != 0 {
            continue;
        }
        let mut q = std::collections::VecDeque::new();
        q.push_back(start);
        labels[start] = next_label;
        while let Some(vi) = q.pop_front() {
            let iz = vi / (dims[0] * dims[1]);
            let rem = vi % (dims[0] * dims[1]);
            let iy = rem / dims[0];
            let ix = rem % dims[0];
            let neighbors = [
                if ix > 0 {
                    Some(idx(ix - 1, iy, iz))
                } else {
                    None
                },
                if ix + 1 < dims[0] {
                    Some(idx(ix + 1, iy, iz))
                } else {
                    None
                },
                if iy > 0 {
                    Some(idx(ix, iy - 1, iz))
                } else {
                    None
                },
                if iy + 1 < dims[1] {
                    Some(idx(ix, iy + 1, iz))
                } else {
                    None
                },
                if iz > 0 {
                    Some(idx(ix, iy, iz - 1))
                } else {
                    None
                },
                if iz + 1 < dims[2] {
                    Some(idx(ix, iy, iz + 1))
                } else {
                    None
                },
            ];
            for ni in neighbors.into_iter().flatten() {
                if fg[ni] && labels[ni] == 0 {
                    labels[ni] = next_label;
                    q.push_back(ni);
                }
            }
        }
        next_label += 1;
    }

    let label_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Labels", label_data, 1,
        )));
    (result, (next_label - 1) as usize)
}

/// Get component sizes (voxel counts) sorted descending.
pub fn component_sizes(image: &ImageData) -> Vec<(i64, usize)> {
    let arr = match image.point_data().get_array("Labels") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        let l = buf[0] as i64;
        if l > 0 {
            *counts.entry(l).or_insert(0) += 1;
        }
    }
    let mut sizes: Vec<(i64, usize)> = counts.into_iter().collect();
    sizes.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sizes
}

/// Keep only the N largest components, zeroing smaller ones.
pub fn keep_n_largest(image: &ImageData, n: usize) -> ImageData {
    let sizes = component_sizes(image);
    let keep: std::collections::HashSet<i64> = sizes.iter().take(n).map(|&(l, _)| l).collect();
    let arr = match image.point_data().get_array("Labels") {
        Some(a) => a,
        None => return image.clone(),
    };
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if keep.contains(&(buf[0] as i64)) {
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Labels", data, 1)));
    result
}

/// Component statistics as a Table.
pub fn component_stats_table(image: &ImageData) -> Table {
    let sizes = component_sizes(image);
    let labels: Vec<f64> = sizes.iter().map(|&(l, _)| l as f64).collect();
    let counts: Vec<f64> = sizes.iter().map(|&(_, c)| c as f64).collect();
    let sp = image.spacing();
    let voxel_vol = sp[0] * sp[1] * sp[2];
    let volumes: Vec<f64> = counts.iter().map(|&c| c * voxel_vol).collect();

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "ComponentId",
            labels,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "VoxelCount",
            counts,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Volume", volumes, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn label_two() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "m",
            |x, y, _| {
                if (x < 4.0 && y < 4.0) || (x > 6.0 && y > 6.0) {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let (result, nc) = label_components_3d(&img, "m");
        assert_eq!(nc, 2);
        assert!(result.point_data().get_array("Labels").is_some());
    }
    #[test]
    fn includes_threshold_edge_and_first_component() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![0.5, 2.0, 0.0, 0.0, 2.0, 9.0],
                2,
            )));
        let (result, nc) = label_components_3d(&img, "m");
        assert_eq!(nc, 2);
        let labels = result.point_data().get_array("Labels").unwrap();
        let mut buf = [0.0f64];
        labels.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        labels.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
        labels.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 2.0);
    }
    #[test]
    fn sizes() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "m",
            |x, _, _| if x < 5.0 { 1.0 } else { 0.0 },
        );
        let (labeled, _) = label_components_3d(&img, "m");
        let s = component_sizes(&labeled);
        assert!(!s.is_empty());
    }
    #[test]
    fn equal_sizes_keep_label_order() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Labels",
                vec![2.0, 0.0, 1.0, 0.0, 3.0],
                1,
            )));
        assert_eq!(component_sizes(&img), vec![(1, 1), (2, 1), (3, 1)]);
    }
    #[test]
    fn keep_largest() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "m",
            |x, y, _| {
                if (x < 3.0 && y < 3.0) || (x > 5.0 && y > 5.0) {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let (labeled, _) = label_components_3d(&img, "m");
        let filtered = keep_n_largest(&labeled, 1);
        let s = component_sizes(&filtered);
        assert!(s.len() <= 1);
    }
    #[test]
    fn stats() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "m",
            |x, _, _| if x < 5.0 { 1.0 } else { 0.0 },
        );
        let (labeled, _) = label_components_3d(&img, "m");
        let table = component_stats_table(&labeled);
        assert!(table.num_rows() >= 1);
    }
}
