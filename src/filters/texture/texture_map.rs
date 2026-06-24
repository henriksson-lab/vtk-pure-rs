use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate texture coordinates by projecting points onto a plane.
///
/// The projection plane is defined by an origin, and two axes (point1, point2).
/// Each point is projected and its (u, v) coordinates are computed relative
/// to these axes.
pub fn texture_map_to_plane(
    input: &PolyData,
    origin: [f64; 3],
    point1: [f64; 3],
    point2: [f64; 3],
) -> PolyData {
    let ax = [
        point1[0] - origin[0],
        point1[1] - origin[1],
        point1[2] - origin[2],
    ];
    let ay = [
        point2[0] - origin[0],
        point2[1] - origin[1],
        point2[2] - origin[2],
    ];

    let ax_len2 = ax[0] * ax[0] + ax[1] * ax[1] + ax[2] * ax[2];
    let ay_len2 = ay[0] * ay[0] + ay[1] * ay[1] + ay[2] * ay[2];

    let mut tcoords = DataArray::<f64>::new("TCoords", 2);

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];

        let u = if ax_len2 > 1e-20 {
            (d[0] * ax[0] + d[1] * ax[1] + d[2] * ax[2]) / ax_len2
        } else {
            0.0
        };
        let v = if ay_len2 > 1e-20 {
            (d[0] * ay[0] + d[1] * ay[1] + d[2] * ay[2]) / ay_len2
        } else {
            0.0
        };

        tcoords.push_tuple(&[u, v]);
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

/// Generate texture coordinates by mapping points to spherical coordinates.
///
/// Maps (theta, phi) of each point relative to `center` to (u, v) in [0, 1].
pub fn texture_map_to_sphere(input: &PolyData, center: [f64; 3]) -> PolyData {
    let mut tcoords = DataArray::<f64>::new("TCoords", 2);

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();

        let theta = if r > 1e-20 { (dz / r).acos() } else { 0.0 };
        let phi = dy.atan2(dx);

        let u = (phi + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;

        tcoords.push_tuple(&[u, v]);
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_mapping() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = texture_map_to_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), 3);

        let mut uv = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut uv);
        assert!((uv[0]).abs() < 1e-10); // origin -> (0,0)
        assert!((uv[1]).abs() < 1e-10);

        tc.tuple_as_f64(1, &mut uv);
        assert!((uv[0] - 1.0).abs() < 1e-10); // point1 -> (1,0)
    }

    #[test]
    fn sphere_mapping() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let result = texture_map_to_sphere(&pd, [0.0, 0.0, 0.0]);
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), 3);

        // North pole (0,0,1) should have v near 0
        let mut uv = [0.0f64; 2];
        tc.tuple_as_f64(2, &mut uv);
        assert!(uv[1] < 0.1);
    }
}
