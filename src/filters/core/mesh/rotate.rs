use crate::data::{Points, PolyData};

/// Rotate all points of a PolyData around an arbitrary axis through a center point
/// using Rodrigues' rotation formula.
///
/// * `axis` — rotation axis (will be normalized internally).
/// * `angle_degrees` — rotation angle in degrees.
/// * `center` — the point the axis passes through.
pub fn rotate_axis_angle(
    input: &PolyData,
    axis: [f64; 3],
    angle_degrees: f64,
    center: [f64; 3],
) -> PolyData {
    // Normalize axis
    let len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-15 {
        return input.clone();
    }
    let k: [f64; 3] = [axis[0] / len, axis[1] / len, axis[2] / len];

    let theta: f64 = angle_degrees.to_radians();
    let cos_t: f64 = theta.cos();
    let sin_t: f64 = theta.sin();

    let mut out_points = Points::<f64>::new();
    for i in 0..input.points.len() {
        let p = input.points.get(i);
        // Translate so center is at origin
        let v: [f64; 3] = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];

        // k x v
        let cross: [f64; 3] = [
            k[1] * v[2] - k[2] * v[1],
            k[2] * v[0] - k[0] * v[2],
            k[0] * v[1] - k[1] * v[0],
        ];

        // k . v
        let dot: f64 = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];

        // Rodrigues: v_rot = v*cos(t) + (k x v)*sin(t) + k*(k.v)*(1-cos(t))
        let rotated: [f64; 3] = [
            v[0] * cos_t + cross[0] * sin_t + k[0] * dot * (1.0 - cos_t) + center[0],
            v[1] * cos_t + cross[1] * sin_t + k[1] * dot * (1.0 - cos_t) + center[1],
            v[2] * cos_t + cross[2] * sin_t + k[2] * dot * (1.0 - cos_t) + center[2],
        ];

        out_points.push(rotated);
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn rotate_360_returns_to_original() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let result = rotate_axis_angle(&pd, [0.0, 0.0, 1.0], 360.0, [0.0, 0.0, 0.0]);
        for i in 0..pd.points.len() {
            let orig = pd.points.get(i);
            let rot = result.points.get(i);
            assert!((orig[0] - rot[0]).abs() < 1e-10);
            assert!((orig[1] - rot[1]).abs() < 1e-10);
            assert!((orig[2] - rot[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn rotate_90_around_z() {
        // Rotate (1,0,0) 90 degrees around Z => (0,1,0)
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = rotate_axis_angle(&pd, [0.0, 0.0, 1.0], 90.0, [0.0, 0.0, 0.0]);
        let p0 = result.points.get(0);
        assert!(p0[0].abs() < 1e-10);
        assert!((p0[1] - 1.0).abs() < 1e-10);
        assert!(p0[2].abs() < 1e-10);
    }

    #[test]
    fn rotate_around_non_origin_center() {
        // Rotate (2,0,0) 180 degrees around Z centered at (1,0,0) => (0,0,0)
        let pd = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = rotate_axis_angle(&pd, [0.0, 0.0, 1.0], 180.0, [1.0, 0.0, 0.0]);
        let p0 = result.points.get(0);
        assert!(p0[0].abs() < 1e-10);
        assert!(p0[1].abs() < 1e-10);
    }

    #[test]
    fn rotate_preserves_attributes() {
        let mut pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(DataArray::from_vec("ids", vec![1.0, 2.0, 3.0], 1).into());

        let result = rotate_axis_angle(&pd, [0.0, 0.0, 1.0], 90.0, [0.0, 0.0, 0.0]);

        assert!(result.point_data().get_array("ids").is_some());
    }
}
