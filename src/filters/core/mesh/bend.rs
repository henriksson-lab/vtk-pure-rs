use crate::data::PolyData;

/// Bend a mesh around an axis.
///
/// Each vertex is rotated by an angle proportional to its signed distance
/// along `axis` (measured from `center`). The rotation is performed around
/// `bend_axis` passing through `center`.
///
/// `angle_per_unit` controls how many radians of bend are applied per unit
/// distance along the primary axis.
pub fn bend_mesh(
    input: &PolyData,
    axis: [f64; 3],
    center: [f64; 3],
    bend_axis: [f64; 3],
    angle_per_unit: f64,
) -> PolyData {
    let mut pd = input.clone();

    // Normalize axis
    let a_len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if a_len < 1e-20 {
        return pd;
    }
    let ax: [f64; 3] = [axis[0] / a_len, axis[1] / a_len, axis[2] / a_len];

    // Normalize bend_axis
    let b_len: f64 =
        (bend_axis[0] * bend_axis[0] + bend_axis[1] * bend_axis[1] + bend_axis[2] * bend_axis[2])
            .sqrt();
    if b_len < 1e-20 {
        return pd;
    }
    let bx: [f64; 3] = [
        bend_axis[0] / b_len,
        bend_axis[1] / b_len,
        bend_axis[2] / b_len,
    ];

    for i in 0..pd.points.len() {
        let p = pd.points.get(i);
        let dx: f64 = p[0] - center[0];
        let dy: f64 = p[1] - center[1];
        let dz: f64 = p[2] - center[2];

        // Distance along axis
        let dist_along: f64 = dx * ax[0] + dy * ax[1] + dz * ax[2];
        let angle: f64 = dist_along * angle_per_unit;

        // Rodrigues' rotation around bend_axis through center
        let cos_a: f64 = angle.cos();
        let sin_a: f64 = angle.sin();

        // v_dot_k = dot(d, bend_axis)
        let v_dot_k: f64 = dx * bx[0] + dy * bx[1] + dz * bx[2];

        // k_cross_v = bend_axis x d
        let kxv: [f64; 3] = [
            bx[1] * dz - bx[2] * dy,
            bx[2] * dx - bx[0] * dz,
            bx[0] * dy - bx[1] * dx,
        ];

        // Rodrigues: v_rot = v*cos(a) + (k x v)*sin(a) + k*(k.v)*(1-cos(a))
        let rx: f64 = dx * cos_a + kxv[0] * sin_a + bx[0] * v_dot_k * (1.0 - cos_a);
        let ry: f64 = dy * cos_a + kxv[1] * sin_a + bx[1] * v_dot_k * (1.0 - cos_a);
        let rz: f64 = dz * cos_a + kxv[2] * sin_a + bx[2] * v_dot_k * (1.0 - cos_a);

        pd.points
            .set(i, [center[0] + rx, center[1] + ry, center[2] + rz]);
    }

    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_angle_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        let result = bend_mesh(&pd, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.0);
        for i in 0..pd.points.len() {
            let orig = pd.points.get(i);
            let bent = result.points.get(i);
            assert!((orig[0] - bent[0]).abs() < 1e-10);
            assert!((orig[1] - bent[1]).abs() < 1e-10);
            assert!((orig[2] - bent[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn bend_along_x_around_z() {
        // Points along x-axis, bending around z-axis
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // at center, no rotation
        pd.points.push([1.0, 0.0, 0.0]); // 1 unit along x

        let angle_per_unit: f64 = std::f64::consts::FRAC_PI_2; // 90 degrees per unit
        let result = bend_mesh(
            &pd,
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            angle_per_unit,
        );

        // Center point should not move
        let p0 = result.points.get(0);
        assert!((p0[0]).abs() < 1e-10);
        assert!((p0[1]).abs() < 1e-10);

        // Point at x=1 rotated 90 degrees around z: (1,0,0) -> (0,1,0)
        let p1 = result.points.get(1);
        assert!((p1[0]).abs() < 1e-10, "expected ~0, got {}", p1[0]);
        assert!((p1[1] - 1.0).abs() < 1e-10, "expected ~1, got {}", p1[1]);
    }

    #[test]
    fn negative_distance_bends_opposite() {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, 0.0, 0.0]); // -1 along x

        let angle_per_unit: f64 = std::f64::consts::FRAC_PI_2;
        let result = bend_mesh(
            &pd,
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            angle_per_unit,
        );

        // (-1,0,0) rotated -90 deg around z -> (0,1,0)
        let p = result.points.get(0);
        assert!((p[0]).abs() < 1e-10, "expected ~0, got {}", p[0]);
        assert!((p[1] - 1.0).abs() < 1e-10, "expected ~1, got {}", p[1]);
    }
}
