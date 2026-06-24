use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Voxel-grid downsampling of a point cloud.
///
/// Divides space into a grid of `cell_size` and keeps one point per cell
/// (the centroid of all points in that cell). Uniform density reduction.
pub fn voxel_downsample(input: &PolyData, cell_size: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || cell_size <= 0.0 {
        return input.clone();
    }
    let inv = 1.0 / cell_size;

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

/// Random downsampling: keep a fraction of points uniformly at random.
pub fn random_downsample(input: &PolyData, fraction: f64, seed: u64) -> PolyData {
    let n = input.points.len();
    let frac = fraction.clamp(0.0, 1.0);

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut rng = seed;

    for i in 0..n {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = (rng >> 33) as f64 / (1u64 << 31) as f64;
        if r < frac {
            let idx = out_points.len() as i64;
            out_points.push(input.points.get(i));
            out_verts.push_cell(&[idx]);
        }
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
    fn voxel_reduces() {
        let mut pd = PolyData::new();
        for i in 0..100 {
            pd.points.push([(i % 10) as f64 * 0.01, 0.0, 0.0]);
        }
        let result = voxel_downsample(&pd, 0.05);
        assert!(result.points.len() < 100);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn voxel_distinct_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        let result = voxel_downsample(&pd, 1.0);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn random_fraction() {
        let mut pd = PolyData::new();
        for i in 0..1000 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let result = random_downsample(&pd, 0.1, 42);
        assert!(result.points.len() > 50);
        assert!(result.points.len() < 200);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(voxel_downsample(&pd, 1.0).points.len(), 0);
        assert_eq!(random_downsample(&pd, 0.5, 0).points.len(), 0);
    }
}
