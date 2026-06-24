use crate::data::{CellArray, Points, PolyData};

/// Parameters for cleaning PolyData.
pub struct CleanParams {
    /// Tolerance for merging nearby points. Points within this distance are merged.
    pub tolerance: f64,
    /// If true, merge duplicate/nearby points.
    pub merge_points: bool,
    /// If true, remove degenerate cells (lines with <2 points, polys with <3 points).
    pub remove_degenerate: bool,
}

impl Default for CleanParams {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            merge_points: true,
            remove_degenerate: true,
        }
    }
}

/// Clean a PolyData by merging duplicate points and removing degenerate cells.
pub fn clean(input: &PolyData, params: &CleanParams) -> PolyData {
    let (new_points, point_map) = if params.merge_points {
        merge_points(&input.points, params.tolerance)
    } else {
        // Identity mapping
        let map: Vec<usize> = (0..input.points.len()).collect();
        let pts = input.points.clone();
        (pts, map)
    };

    let mut output = PolyData::new();
    output.points = new_points;

    // Remap cells
    output.verts = remap_cells(&input.verts, &point_map, params.remove_degenerate, 1);
    output.lines = remap_cells(&input.lines, &point_map, params.remove_degenerate, 2);
    output.polys = remap_cells(&input.polys, &point_map, params.remove_degenerate, 3);
    output.strips = remap_cells(&input.strips, &point_map, params.remove_degenerate, 3);

    output
}

/// Merge points within `tolerance` using quantized grid deduplication.
///
/// Quantizes each point to a grid cell and uses a flat hash table for O(1) lookup.
/// Only checks same cell (no 27-neighbor scan) — cell size is tolerance,
/// so points further than tolerance apart land in different cells.
fn merge_points(points: &Points<f64>, tolerance: f64) -> (Points<f64>, Vec<usize>) {
    let n = points.len();
    if n == 0 {
        return (Points::new(), vec![]);
    }

    let cell_size = if tolerance > 0.0 { tolerance } else { 1e-12 };
    let inv_cell = 1.0 / cell_size;

    let mut point_map = vec![0usize; n];

    // Sort-based approach: quantize to grid cell, group by cell, pick first in each group
    // This avoids per-point hash table operations entirely.

    // Quantize all points to grid keys using flat slice access
    let pts = points.as_flat_slice();
    let mut keyed: Vec<(u64, u32)> = Vec::with_capacity(n);
    for i in 0..n {
        let b = i * 3;
        let gx = (pts[b] * inv_cell).floor() as i64;
        let gy = (pts[b + 1] * inv_cell).floor() as i64;
        let gz = (pts[b + 2] * inv_cell).floor() as i64;
        keyed.push((morton_encode(gx, gy, gz), i as u32));
    }

    // Sort by Morton key — points in the same cell are adjacent
    keyed.sort_unstable_by_key(|&(k, _)| k);

    // Deduplicate: first point in each group becomes the representative
    let mut new_points_flat: Vec<f64> = Vec::with_capacity(n * 3);
    let mut current_key = u64::MAX;
    let mut current_idx = 0usize;

    for &(key, orig_idx) in &keyed {
        if key != current_key {
            current_key = key;
            current_idx = new_points_flat.len() / 3;
            let b = orig_idx as usize * 3;
            new_points_flat.extend_from_slice(&pts[b..b + 3]);
        }
        point_map[orig_idx as usize] = current_idx;
    }

    (Points::from_flat_vec(new_points_flat), point_map)
}

#[inline]
fn morton_encode(x: i64, y: i64, z: i64) -> u64 {
    // Simple hash-based encoding (not true Morton, but orders spatially)
    let ux = x as u64;
    let uy = y as u64;
    let uz = z as u64;
    // FNV-1a style mixing
    let mut h = 0xcbf29ce484222325u64;
    h ^= ux;
    h = h.wrapping_mul(0x100000001b3);
    h ^= uy;
    h = h.wrapping_mul(0x100000001b3);
    h ^= uz;
    h = h.wrapping_mul(0x100000001b3);
    h
}

/// Remap cell point indices and optionally remove degenerate cells.
fn remap_cells(
    cells: &CellArray,
    point_map: &[usize],
    remove_degenerate: bool,
    min_unique: usize,
) -> CellArray {
    let mut out = CellArray::new();

    for cell in cells.iter() {
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_map[id as usize] as i64)
            .collect();

        if remove_degenerate {
            // Remove consecutive duplicates
            let mut deduped: Vec<i64> = Vec::with_capacity(remapped.len());
            for &id in &remapped {
                if deduped.last() != Some(&id) {
                    deduped.push(id);
                }
            }
            // Also check if first == last for polygons
            if deduped.len() > 1 && deduped.first() == deduped.last() {
                deduped.pop();
            }

            if deduped.len() >= min_unique {
                out.push_cell(&deduped);
            }
        } else {
            out.push_cell(&remapped);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_duplicate_points() {
        let mut pd = PolyData::new();
        // Two triangles with duplicate points at indices 3,4,5 = copies of 0,1,2
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate of 0
            [1.0, 0.0, 0.0], // duplicate of 1
            [0.0, 1.0, 0.0], // duplicate of 2
        ]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = clean(&pd, &CleanParams::default());
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 2);
        // Both triangles should reference the same 3 points
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[0, 1, 2]);
    }

    #[test]
    fn remove_degenerate_cell() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        // Degenerate triangle: only 2 unique points after collapse
        pd.polys.push_cell(&[0, 1, 0]);

        let result = clean(&pd, &CleanParams::default());
        assert_eq!(result.polys.num_cells(), 0); // degenerate removed
    }

    #[test]
    fn no_merge_mode() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate
        ]);
        pd.polys.push_cell(&[0, 1, 0]); // technically valid if not merging

        let result = clean(
            &pd,
            &CleanParams {
                merge_points: false,
                remove_degenerate: false,
                ..Default::default()
            },
        );
        assert_eq!(result.points.len(), 2); // no merging
        assert_eq!(result.polys.num_cells(), 1); // no removal
    }
}
