//! PoissonDiskSampler — Poisson disk subsampling from existing points.

use crate::data::{CellArray, Points, PolyData};

/// Poisson disk subsampling using dart-throwing.
///
/// Iterates through points in random order (seeded by `seed`), accepting
/// each point only if no previously accepted point lies within `radius`.
/// Returns a PolyData with the accepted subset as vertex cells.
pub fn poisson_disk_sample(input: &PolyData, radius: f64, seed: u64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let r2 = radius * radius;

    // Simple LCG PRNG for shuffling
    let mut indices: Vec<usize> = (0..n).collect();
    let mut rng_state = seed.wrapping_add(1);
    // Fisher-Yates shuffle
    for i in (1..n).rev() {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng_state >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }

    let mut accepted: Vec<usize> = Vec::new();

    for &idx in &indices {
        let p = pts[idx];
        let mut too_close = false;
        for &aidx in &accepted {
            let q = pts[aidx];
            let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            if d2 < r2 {
                too_close = true;
                break;
            }
        }
        if !too_close {
            accepted.push(idx);
        }
    }

    let mut new_pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for (i, &idx) in accepted.iter().enumerate() {
        new_pts.push(pts[idx]);
        verts.push_cell(&[i as i64]);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.verts = verts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsamples_points() {
        let mut pd = PolyData::new();
        // Dense grid
        for i in 0..10 {
            for j in 0..10 {
                pd.points.push([i as f64 * 0.1, j as f64 * 0.1, 0.0]);
            }
        }

        let result = poisson_disk_sample(&pd, 0.25, 42);
        // Should have fewer points than input
        assert!(result.points.len() < 100);
        assert!(result.points.len() > 0);

        // Verify minimum distance constraint
        let n = result.points.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = result.points.get(i);
                let b = result.points.get(j);
                let d2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
                assert!(
                    d2 >= 0.25 * 0.25 - 1e-10,
                    "points too close: d={}",
                    d2.sqrt()
                );
            }
        }
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = poisson_disk_sample(&pd, 1.0, 0);
        assert_eq!(result.points.len(), 1);
    }
}
