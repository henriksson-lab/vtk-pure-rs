//! Project a surface onto a sphere.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Project all points of a mesh onto a sphere of given radius centered
/// at the given center point.
///
/// Each point is moved radially to lie exactly on the sphere surface.
/// The mesh connectivity is preserved.
pub fn project_to_sphere(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let mut new_points = Points::<f64>::new();
    let mut displacement = Vec::with_capacity(n);

    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 1e-15 {
            // Point at center — project to north pole
            new_points.push([center[0], center[1], center[2] + radius]);
            displacement.push(radius);
        } else {
            let scale = radius / dist;
            new_points.push([
                center[0] + dx * scale,
                center[1] + dy * scale,
                center[2] + dz * scale,
            ]);
            displacement.push((dist - radius).abs());
        }
    }

    let mut result = mesh.clone();
    result.points = new_points;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ProjectionDisplacement",
            displacement,
            1,
        )));
    result
}

/// Project points onto a sphere and add spherical coordinate arrays.
pub fn project_to_sphere_with_coords(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let mut result = project_to_sphere(mesh, center, radius);
    let n = result.points.len();

    let mut theta_data = Vec::with_capacity(n);
    let mut phi_data = Vec::with_capacity(n);

    for i in 0..n {
        let p = result.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let theta = if r > 1e-15 { (dz / r).acos() } else { 0.0 };
        let phi = dy.atan2(dx);
        theta_data.push(theta.to_degrees());
        phi_data.push(phi.to_degrees());
    }

    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Theta", theta_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi_data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_cube_to_sphere() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = project_to_sphere(&mesh, [0.0, 0.0, 0.0], 2.0);
        assert_eq!(result.points.len(), 6);

        // All points should be at distance 2.0 from origin
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 2.0).abs() < 1e-10, "r={r}");
        }
    }

    #[test]
    fn with_coords() {
        let mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        let result = project_to_sphere_with_coords(&mesh, [0.0, 0.0, 0.0], 1.0);
        assert!(result.point_data().get_array("Theta").is_some());
        assert!(result.point_data().get_array("Phi").is_some());
        assert!(result
            .point_data()
            .get_array("ProjectionDisplacement")
            .is_some());
    }

    #[test]
    fn already_on_sphere() {
        let mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0]]);
        let result = project_to_sphere(&mesh, [0.0, 0.0, 0.0], 1.0);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);

        let arr = result
            .point_data()
            .get_array("ProjectionDisplacement")
            .unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10); // no displacement
    }

    #[test]
    fn empty_mesh() {
        let result = project_to_sphere(&PolyData::new(), [0.0, 0.0, 0.0], 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
