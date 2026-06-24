//! Hierarchical binning of point clouds.
//!
//! Organizes points into a multi-level spatial bin hierarchy (octree-like)
//! with configurable depth and bin labeling.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

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
pub fn bin_counts(mesh: &PolyData) -> std::collections::HashMap<usize, usize> {
    let arr = match mesh.point_data().get_array("BinId") {
        Some(a) => a,
        None => return std::collections::HashMap::new(),
    };
    let mut counts = std::collections::HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as usize).or_insert(0) += 1;
    }
    counts
}

/// Extract points belonging to a specific bin ID.
pub fn extract_bin(mesh: &PolyData, bin_id: usize) -> PolyData {
    let arr = match mesh.point_data().get_array("BinId") {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let mut points = Points::<f64>::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] as usize == bin_id {
            points.push(mesh.points.get(i));
        }
    }
    let mut result = PolyData::new();
    result.points = points;
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
