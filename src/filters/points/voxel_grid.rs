use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Voxel-based point cloud downsampling.
///
/// Divides space into a regular grid of voxels with the given `voxel_size`.
/// For each occupied voxel, emits a single representative point at the
/// centroid of all points falling within that voxel.
///
/// Returns a PolyData with vertex cells.
pub fn voxel_grid(input: &PolyData, voxel_size: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || voxel_size <= 0.0 {
        return input.clone();
    }

    let inv = 1.0 / voxel_size;
    let mut bins: HashMap<(i64, i64, i64), (f64, f64, f64, usize)> = HashMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let key = (
            (p[0] * inv).floor() as i64,
            (p[1] * inv).floor() as i64,
            (p[2] * inv).floor() as i64,
        );
        let entry = bins.entry(key).or_insert((0.0, 0.0, 0.0, 0));
        entry.0 += p[0];
        entry.1 += p[1];
        entry.2 += p[2];
        entry.3 += 1;
    }

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for (_, (sx, sy, sz, c)) in &bins {
        let idx = out_points.len() as i64;
        let nf = *c as f64;
        out_points.push([sx / nf, sy / nf, sz / nf]);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_clusters() {
        let mut pd = PolyData::new();
        // 100 points clustered in 10 voxels
        for i in 0..100 {
            pd.points.push([(i % 10) as f64 * 0.01, 0.0, 0.0]);
        }
        let result = voxel_grid(&pd, 0.05);
        assert!(result.points.len() < 100);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn distinct_voxels() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        let result = voxel_grid(&pd, 1.0);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.verts.num_cells(), 2);
    }

    #[test]
    fn single_voxel_centroid() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.2, 0.0, 0.0]);
        let result = voxel_grid(&pd, 1.0);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(voxel_grid(&pd, 1.0).points.len(), 0);
    }
}
