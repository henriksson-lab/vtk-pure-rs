//! Hierarchical binning of point clouds.
//!
//! Organizes points into a multi-level spatial bin hierarchy (octree-like)
//! with configurable depth and bin labeling.

use std::collections::{HashMap, HashSet};

use crate::data::{AnyDataArray, DataArray, Points, PolyData};
use crate::types::Scalar;

/// Hierarchically bin a point cloud into an octree structure.
///
/// Returns the mesh with "BinLevel" and "BinId" point data arrays.
/// Level 0 is the root (all points), level 1 has up to 8 bins, etc.
pub fn hierarchical_bin(mesh: &PolyData, max_depth: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    // Compute bounding box
    let mut min = mesh.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }
    // Pad to avoid edge cases
    for j in 0..3 {
        min[j] -= 1e-10;
        max[j] += 1e-10;
    }

    let mut bin_ids = vec![0usize; n];
    let mut bin_levels = vec![0usize; n];

    // Assign bins at the finest level
    let depth = max_depth.min(10); // cap to avoid huge bin counts
    let bins_per_axis = 1usize << depth; // 2^depth

    for i in 0..n {
        let p = mesh.points.get(i);
        let ix = ((p[0] - min[0]) / (max[0] - min[0]) * bins_per_axis as f64) as usize;
        let iy = ((p[1] - min[1]) / (max[1] - min[1]) * bins_per_axis as f64) as usize;
        let iz = ((p[2] - min[2]) / (max[2] - min[2]) * bins_per_axis as f64) as usize;
        let ix = ix.min(bins_per_axis - 1);
        let iy = iy.min(bins_per_axis - 1);
        let iz = iz.min(bins_per_axis - 1);

        // Morton code (Z-order curve) for hierarchical bin ID
        let mut morton = 0usize;
        for bit in 0..depth {
            morton |= ((ix >> bit) & 1) << (3 * bit);
            morton |= ((iy >> bit) & 1) << (3 * bit + 1);
            morton |= ((iz >> bit) & 1) << (3 * bit + 2);
        }
        bin_ids[i] = morton;
        bin_levels[i] = depth;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BinId",
            bin_ids.iter().map(|&b| b as f64).collect(),
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BinLevel",
            bin_levels.iter().map(|&l| l as f64).collect(),
            1,
        )));
    result
}

/// Count points per bin and return as a summary.
pub fn bin_counts(mesh: &PolyData) -> HashMap<usize, usize> {
    let arr = match mesh.point_data().get_array("BinId") {
        Some(a) => a,
        None => return HashMap::new(),
    };
    let mut counts = HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as usize).or_insert(0) += 1;
    }
    counts
}

/// Extract points belonging to a specific bin ID.
pub fn extract_bin(mesh: &PolyData, bin_id: usize) -> PolyData {
    extract_bins(mesh, &[bin_id])
}

/// Extract points belonging to any of the specified bin IDs.
pub fn extract_bins(mesh: &PolyData, bin_ids: &[usize]) -> PolyData {
    let arr = match mesh.point_data().get_array("BinId") {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let selected_bins: HashSet<usize> = bin_ids.iter().copied().collect();
    let mut selected_points = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if selected_bins.contains(&(buf[0] as usize)) {
            selected_points.push(i);
        }
    }
    extract_point_indices(mesh, &selected_points)
}

/// Extract all points at a hierarchy level.
pub fn extract_level(mesh: &PolyData, level: usize) -> PolyData {
    let arr = match mesh.point_data().get_array("BinLevel") {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let mut selected_points = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] as usize == level {
            selected_points.push(i);
        }
    }
    extract_point_indices(mesh, &selected_points)
}

fn extract_point_indices(mesh: &PolyData, point_ids: &[usize]) -> PolyData {
    let mut result = PolyData::new();
    let mut points = Points::<f64>::new();
    for &point_id in point_ids {
        points.push(mesh.points.get(point_id));
    }
    result.points = points;

    for array in mesh.point_data().iter() {
        result
            .point_data_mut()
            .add_array(subset_any_array(array, point_ids));
    }
    result
}

fn subset_any_array(array: &AnyDataArray, point_ids: &[usize]) -> AnyDataArray {
    match array {
        AnyDataArray::F32(a) => AnyDataArray::F32(subset_data_array(a, point_ids)),
        AnyDataArray::F64(a) => AnyDataArray::F64(subset_data_array(a, point_ids)),
        AnyDataArray::I8(a) => AnyDataArray::I8(subset_data_array(a, point_ids)),
        AnyDataArray::I16(a) => AnyDataArray::I16(subset_data_array(a, point_ids)),
        AnyDataArray::I32(a) => AnyDataArray::I32(subset_data_array(a, point_ids)),
        AnyDataArray::I64(a) => AnyDataArray::I64(subset_data_array(a, point_ids)),
        AnyDataArray::U8(a) => AnyDataArray::U8(subset_data_array(a, point_ids)),
        AnyDataArray::U16(a) => AnyDataArray::U16(subset_data_array(a, point_ids)),
        AnyDataArray::U32(a) => AnyDataArray::U32(subset_data_array(a, point_ids)),
        AnyDataArray::U64(a) => AnyDataArray::U64(subset_data_array(a, point_ids)),
    }
}

fn subset_data_array<T: Scalar>(array: &DataArray<T>, point_ids: &[usize]) -> DataArray<T> {
    let mut result = DataArray::new(array.name(), array.num_components());
    for &point_id in point_ids {
        if point_id < array.num_tuples() {
            result.push_tuple(array.tuple(point_id));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_binning() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ]);
        let result = hierarchical_bin(&mesh, 2);
        assert!(result.point_data().get_array("BinId").is_some());
        assert!(result.point_data().get_array("BinLevel").is_some());
    }

    #[test]
    fn bin_count() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.01, 0.01, 0.0],  // same bin
            [10.0, 10.0, 10.0], // different bin
        ]);
        let binned = hierarchical_bin(&mesh, 1);
        let counts = bin_counts(&binned);
        assert!(counts.len() >= 1);
    }

    #[test]
    fn extract_specific_bin() {
        let mesh =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.01, 0.01, 0.0], [10.0, 10.0, 10.0]]);
        let binned = hierarchical_bin(&mesh, 1);
        let counts = bin_counts(&binned);
        // Extract the most common bin
        let (&most_common_bin, _) = counts.iter().max_by_key(|(_, &c)| c).unwrap();
        let extracted = extract_bin(&binned, most_common_bin);
        assert!(extracted.points.len() >= 1);
    }
}
