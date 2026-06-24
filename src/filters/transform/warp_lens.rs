//! WarpLens — apply barrel/pincushion lens distortion to mesh points.

use crate::data::PolyData;

/// Apply barrel (k1 > 0) or pincushion (k1 < 0) lens distortion.
///
/// For each point, computes distance `r` from `center` (in XY plane),
/// then applies `r' = r * (1 + k1 * r^2)` and scales the point position
/// relative to center by `r'/r`.
pub fn warp_lens(input: &PolyData, center: [f64; 2], k1: f64) -> PolyData {
    let mut output = input.clone();
    let n = input.points.len();

    for i in 0..n {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let r2 = dx * dx + dy * dy;
        let r = r2.sqrt();

        if r < 1e-15 {
            continue;
        }

        let r_prime = r * (1.0 + k1 * r2);
        let scale = r_prime / r;

        output
            .points
            .set(i, [center[0] + dx * scale, center[1] + dy * scale, p[2]]);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barrel_distortion_moves_outward() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        pd.points.push([1.0, 0.0, 0.0]); // r=1

        let result = warp_lens(&pd, [0.0, 0.0], 0.1);
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        // Center should not move
        assert!((p0[0]).abs() < 1e-10);
        // Point at r=1 should move outward with positive k1
        assert!(p1[0] > 1.0);
    }

    #[test]
    fn pincushion_distortion_moves_inward() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);

        let result = warp_lens(&pd, [0.0, 0.0], -0.1);
        let p = result.points.get(0);
        assert!(p[0] < 1.0);
    }

    #[test]
    fn z_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 1.0, 5.0]);

        let result = warp_lens(&pd, [0.0, 0.0], 0.5);
        let p = result.points.get(0);
        assert_eq!(p[2], 5.0);
    }
}
