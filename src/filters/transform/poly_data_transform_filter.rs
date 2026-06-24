use crate::data::{Points, PolyData};

/// Translate all points by a vector.
pub fn translate(input: &PolyData, delta: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([p[0] + delta[0], p[1] + delta[1], p[2] + delta[2]]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Scale all points relative to a center.
pub fn scale(input: &PolyData, factors: [f64; 3], center: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([
            center[0] + (p[0] - center[0]) * factors[0],
            center[1] + (p[1] - center[1]) * factors[1],
            center[2] + (p[2] - center[2]) * factors[2],
        ]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Rotate all points around an axis by angle (degrees).
pub fn rotate(input: &PolyData, axis: [f64; 3], angle_deg: f64, center: [f64; 3]) -> PolyData {
    let angle = angle_deg.to_radians();
    let c = angle.cos();
    let s = angle.sin();
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-15 {
        return input.clone();
    }
    let ux = axis[0] / len;
    let uy = axis[1] / len;
    let uz = axis[2] / len;

    // Rodrigues' rotation formula as matrix
    let r = [
        [
            c + ux * ux * (1.0 - c),
            ux * uy * (1.0 - c) - uz * s,
            ux * uz * (1.0 - c) + uy * s,
        ],
        [
            uy * ux * (1.0 - c) + uz * s,
            c + uy * uy * (1.0 - c),
            uy * uz * (1.0 - c) - ux * s,
        ],
        [
            uz * ux * (1.0 - c) - uy * s,
            uz * uy * (1.0 - c) + ux * s,
            c + uz * uz * (1.0 - c),
        ],
    ];

    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        points.push([
            center[0] + r[0][0] * dx + r[0][1] * dy + r[0][2] * dz,
            center[1] + r[1][0] * dx + r[1][1] * dy + r[1][2] * dz,
            center[2] + r[2][0] * dx + r[2][1] * dy + r[2][2] * dz,
        ]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_points() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = translate(&pd, [10.0, 20.0, 30.0]);
        assert_eq!(result.points.get(0), [11.0, 22.0, 33.0]);
    }

    #[test]
    fn scale_from_origin() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = scale(&pd, [2.0, 2.0, 2.0], [0.0, 0.0, 0.0]);
        assert_eq!(result.points.get(0), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn rotate_90_z() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        let result = rotate(&pd, [0.0, 0.0, 1.0], 90.0, [0.0, 0.0, 0.0]);
        let p = result.points.get(0);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn identity_rotation() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = rotate(&pd, [0.0, 0.0, 1.0], 0.0, [0.0, 0.0, 0.0]);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
        assert!((p[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn scale_from_center() {
        let mut pd = PolyData::new();
        pd.points.push([2.0, 0.0, 0.0]);
        let result = scale(&pd, [3.0, 1.0, 1.0], [1.0, 0.0, 0.0]);
        // (2-1)*3 + 1 = 4
        assert!((result.points.get(0)[0] - 4.0).abs() < 1e-10);
    }
}
