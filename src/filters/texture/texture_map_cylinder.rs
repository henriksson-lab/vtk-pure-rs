//! Cylindrical texture coordinate generation.
//!
//! Maps points to cylindrical (theta, height) coordinates for texture mapping.
//! Analogous to VTK's vtkTextureMapToCylinder.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate cylindrical texture coordinates for a mesh.
///
/// The cylinder axis is defined by two points. Each vertex is projected
/// onto the cylinder, and (u, v) is computed as (theta/2π, t) where
/// theta is the angle around the axis and t is the normalized height.
pub fn texture_map_to_cylinder(
    input: &PolyData,
    axis_point1: [f64; 3],
    axis_point2: [f64; 3],
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let axis = [
        axis_point2[0] - axis_point1[0],
        axis_point2[1] - axis_point1[1],
        axis_point2[2] - axis_point1[2],
    ];
    let axis_len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if axis_len < 1e-15 {
        return input.clone();
    }
    let axis_norm = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];

    // Build local coordinate frame
    // Find a vector not parallel to axis
    let up = if axis_norm[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u_dir = cross(axis_norm, up);
    let u_len = (u_dir[0] * u_dir[0] + u_dir[1] * u_dir[1] + u_dir[2] * u_dir[2]).sqrt();
    let u_dir = [u_dir[0] / u_len, u_dir[1] / u_len, u_dir[2] / u_len];
    let v_dir = cross(axis_norm, u_dir);

    let mut tcoords = DataArray::<f64>::new("TCoords", 2);

    for i in 0..n {
        let p = input.points.get(i);
        let d = [
            p[0] - axis_point1[0],
            p[1] - axis_point1[1],
            p[2] - axis_point1[2],
        ];

        // Project onto axis for height parameter
        let t = dot(d, axis_norm) / axis_len;

        // Project perpendicular to axis for angle
        let proj_u = dot(d, u_dir);
        let proj_v = dot(d, v_dir);
        let theta = proj_v.atan2(proj_u);

        let u = (theta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = t.clamp(0.0, 1.0);

        tcoords.push_tuple(&[u, v]);
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(tcoords));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

/// Generate cylindrical texture coordinates with automatic axis detection.
///
/// Uses the longest axis of the bounding box as the cylinder axis.
pub fn texture_map_to_cylinder_auto(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Find bounding box
    let mut min = input.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = input.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }

    let extents = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let longest = if extents[0] >= extents[1] && extents[0] >= extents[2] {
        0
    } else if extents[1] >= extents[2] {
        1
    } else {
        2
    };

    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];
    let mut p1 = center;
    let mut p2 = center;
    p1[longest] = min[longest];
    p2[longest] = max[longest];

    texture_map_to_cylinder(input, p1, p2)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cylinder_tex_coords() {
        // Points on a cylinder around Y axis
        let mut pts = Vec::new();
        for i in 0..8 {
            let theta = i as f64 * std::f64::consts::PI / 4.0;
            pts.push([theta.cos(), 0.0, theta.sin()]);
            pts.push([theta.cos(), 1.0, theta.sin()]);
        }
        let mesh = PolyData::from_points(pts);

        let result = texture_map_to_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), mesh.points.len());
        assert_eq!(tc.num_components(), 2);

        // Check v coordinates: bottom points should be ~0, top ~1
        let mut buf = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut buf);
        assert!(buf[1] < 0.1, "bottom v={}", buf[1]); // y=0 → v≈0
        tc.tuple_as_f64(1, &mut buf);
        assert!(buf[1] > 0.9, "top v={}", buf[1]); // y=1 → v≈1
    }

    #[test]
    fn auto_cylinder() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.0, 5.0, 0.0], [1.0, 2.5, 0.0]]);
        let result = texture_map_to_cylinder_auto(&mesh);
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        let result = texture_map_to_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(result.points.len(), 0);
    }
}
