use crate::data::PolyData;

/// Taper (scale) a mesh progressively along an axis.
///
/// Vertices are scaled in the plane perpendicular to `axis` by a factor that
/// varies linearly with signed distance from `center` along `axis`.
/// At `center` the scale is 1.0; at distance `d` the scale is
/// `1.0 + d * taper_factor`.
pub fn taper_mesh(
    input: &PolyData,
    axis: [f64; 3],
    center: [f64; 3],
    taper_factor: f64,
) -> PolyData {
    let mut pd = input.clone();

    // Normalize axis
    let a_len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if a_len < 1e-20 {
        return pd;
    }
    let ax: [f64; 3] = [axis[0] / a_len, axis[1] / a_len, axis[2] / a_len];

    for i in 0..pd.points.len() {
        let p = pd.points.get(i);
        let dx: f64 = p[0] - center[0];
        let dy: f64 = p[1] - center[1];
        let dz: f64 = p[2] - center[2];

        // Signed distance along axis
        let dist_along: f64 = dx * ax[0] + dy * ax[1] + dz * ax[2];

        // Component along axis
        let along: [f64; 3] = [dist_along * ax[0], dist_along * ax[1], dist_along * ax[2]];

        // Component perpendicular to axis
        let perp: [f64; 3] = [dx - along[0], dy - along[1], dz - along[2]];

        // Scale factor at this distance
        let scale: f64 = 1.0 + dist_along * taper_factor;
        let scale: f64 = scale.max(0.0); // clamp to avoid inversion

        // Reconstruct: center + along + scale * perp
        pd.points.set(
            i,
            [
                center[0] + along[0] + perp[0] * scale,
                center[1] + along[1] + perp[1] * scale,
                center[2] + along[2] + perp[2] * scale,
            ],
        );
    }

    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_taper_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        let result = taper_mesh(&pd, [0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.0);
        for i in 0..pd.points.len() {
            let orig = pd.points.get(i);
            let tapered = result.points.get(i);
            assert!((orig[0] - tapered[0]).abs() < 1e-10);
            assert!((orig[1] - tapered[1]).abs() < 1e-10);
            assert!((orig[2] - tapered[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn taper_along_z() {
        // Point at (1, 0, 2): along z by 2, perpendicular = (1, 0, 0)
        // taper_factor=0.5 => scale at d=2 is 1+2*0.5 = 2.0
        // result: (0,0,0) + (0,0,2) + 2.0*(1,0,0) = (2,0,2)
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 2.0]);
        let result = taper_mesh(&pd, [0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.5);
        let p = result.points.get(0);
        assert!((p[0] - 2.0).abs() < 1e-10, "expected x=2, got {}", p[0]);
        assert!((p[1]).abs() < 1e-10);
        assert!((p[2] - 2.0).abs() < 1e-10, "expected z=2, got {}", p[2]);
    }

    #[test]
    fn negative_taper_shrinks() {
        // Point at (1, 0, 1): along z by 1, perp = (1, 0, 0)
        // taper_factor=-0.5 => scale at d=1 is 1+1*(-0.5) = 0.5
        // result: (0,0,0) + (0,0,1) + 0.5*(1,0,0) = (0.5,0,1)
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 1.0]);
        let result = taper_mesh(&pd, [0.0, 0.0, 1.0], [0.0, 0.0, 0.0], -0.5);
        let p = result.points.get(0);
        assert!((p[0] - 0.5).abs() < 1e-10, "expected x=0.5, got {}", p[0]);
        assert!((p[2] - 1.0).abs() < 1e-10, "expected z=1, got {}", p[2]);
    }
}
