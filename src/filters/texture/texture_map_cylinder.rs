//! Cylindrical texture coordinate generation.
//!
//! Maps points to cylindrical (theta, height) coordinates for texture mapping.
//! Analogous to VTK's vtkTextureMapToCylinder.

use crate::data::{obb_tree::Obb, AnyDataArray, DataArray, PolyData};

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
    texture_map_to_cylinder_with_prevent_seam(input, axis_point1, axis_point2, true)
}

/// Generate cylindrical texture coordinates with VTK's `PreventSeam` option.
pub fn texture_map_to_cylinder_with_prevent_seam(
    input: &PolyData,
    axis_point1: [f64; 3],
    axis_point2: [f64; 3],
    prevent_seam: bool,
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
    let axis_len2 = dot(axis, axis);
    if axis_len2 == 0.0 {
        return input.clone();
    }

    let mut v = [1.0, 0.0, 0.0];
    let mut vp = cross(axis, v);
    if norm(vp) == 0.0 {
        v = [0.0, 1.0, 0.0];
        vp = cross(axis, v);
    }
    let mut vec = cross(vp, axis);
    if normalize(&mut vec) == 0.0 {
        return input.clone();
    }

    let mut tcoords = DataArray::<f64>::new("TCoords", 2);

    for i in 0..n {
        let p = input.points.get(i);
        let mut d = [
            p[0] - axis_point1[0],
            p[1] - axis_point1[1],
            p[2] - axis_point1[2],
        ];
        let t = dot(d, axis) / axis_len2;
        let closest_t = t.clamp(0.0, 1.0);
        let closest = [
            axis_point1[0] + closest_t * axis[0],
            axis_point1[1] + closest_t * axis[1],
            axis_point1[2] + closest_t * axis[2],
        ];

        for j in 0..3 {
            d[j] = p[j] - closest[j];
        }
        normalize(&mut d);

        let theta_x = dot(d, vec).clamp(-1.0, 1.0).acos();
        let vp = cross(vec, d);
        let theta_y = dot(axis, vp);
        let u = if prevent_seam {
            theta_x / std::f64::consts::PI
        } else {
            let mut u = theta_x / (2.0 * std::f64::consts::PI);
            if theta_y < 0.0 {
                u = 1.0 - u;
            }
            u
        };

        tcoords.push_tuple(&[u, t]);
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
/// Uses the primary axis of the oriented bounding box as the cylinder axis.
pub fn texture_map_to_cylinder_auto(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let points: Vec<[f64; 3]> = input.points.iter().collect();
    let obb = Obb::from_points(&points);
    let axis = obb.axes[0];
    let half_extent = obb.half_extents[0];
    let p1 = [
        obb.center[0] - axis[0] * half_extent,
        obb.center[1] - axis[1] * half_extent,
        obb.center[2] - axis[2] * half_extent,
    ];
    let p2 = [
        obb.center[0] + axis[0] * half_extent,
        obb.center[1] + axis[1] * half_extent,
        obb.center[2] + axis[2] * half_extent,
    ];

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

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn normalize(a: &mut [f64; 3]) -> f64 {
    let len = norm(*a);
    if len != 0.0 {
        a[0] /= len;
        a[1] /= len;
        a[2] /= len;
    }
    len
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
