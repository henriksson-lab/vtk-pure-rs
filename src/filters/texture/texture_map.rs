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

    let mut ax_len2 = ax[0] * ax[0] + ax[1] * ax[1] + ax[2] * ax[2];
    let mut ay_len2 = ay[0] * ay[0] + ay[1] * ay[1] + ay[2] * ay[2];
    if ax_len2 == 0.0 || ay_len2 == 0.0 {
        ax_len2 = 1.0;
        ay_len2 = 1.0;
    }

    let mut tcoords = DataArray::<f64>::new("TCoords", 2);

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];

        let u = (d[0] * ax[0] + d[1] * ax[1] + d[2] * ax[2]) / ax_len2;
        let v = (d[0] * ay[0] + d[1] * ay[1] + d[2] * ay[2]) / ay_len2;

        tcoords.push_tuple(&[u, v]);
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

/// Generate texture coordinates by mapping points to spherical coordinates.
///
/// Matches VTK's manual-center `vtkTextureMapToSphere` default
/// `PreventSeam` mapping.
pub fn texture_map_to_sphere(input: &PolyData, center: [f64; 3]) -> PolyData {
    texture_map_to_sphere_with_prevent_seam(input, center, true)
}

/// Generate spherical texture coordinates with VTK's `PreventSeam` option.
pub fn texture_map_to_sphere_with_prevent_seam(
    input: &PolyData,
    center: [f64; 3],
    prevent_seam: bool,
) -> PolyData {
    if input.points.len() < 1 {
        return copy_poly_data_structure(input);
    }

    let mut tcoords = DataArray::<f64>::new("TCoords", 2);
    let pi = std::f64::consts::PI;
    let pi_over_two = pi / 2.0;

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let rho = distance2_between_points(p, center).sqrt();
        let mut phi = 0.0;
        let mut tc = [0.0f64; 2];

        if rho != 0.0 {
            let diff = p[2] - center[2];
            if diff.abs() > rho {
                if diff > 0.0 {
                    tc[1] = 0.0;
                } else {
                    tc[1] = 1.0;
                }
            } else {
                phi = (diff / rho).acos();
                tc[1] = phi / pi;
            }
        } else {
            tc[1] = 0.0;
        }

        let r = rho * phi.sin();
        let (theta_x, theta_y) = if r != 0.0 {
            let diff = p[0] - center[0];
            let theta_x = if diff.abs() > r {
                if diff > 0.0 {
                    0.0
                } else {
                    pi
                }
            } else {
                (diff / r).acos()
            };

            let diff = p[1] - center[1];
            let theta_y = if diff.abs() > r {
                if diff > 0.0 {
                    pi_over_two
                } else {
                    -pi_over_two
                }
            } else {
                (diff / r).asin()
            };

            (theta_x, theta_y)
        } else {
            (0.0, 0.0)
        };

        if prevent_seam {
            tc[0] = theta_x / pi;
        } else {
            tc[0] = theta_x / (2.0 * pi);
            if theta_y < 0.0 {
                tc[0] = 1.0 - tc[0];
            }
        }
        tcoords.push_tuple(&tc);
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

fn copy_poly_data_structure(input: &PolyData) -> PolyData {
    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.verts = input.verts.clone();
    pd.lines = input.lines.clone();
    pd.polys = input.polys.clone();
    pd.strips = input.strips.clone();
    pd
}

fn distance2_between_points(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
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
    fn plane_mapping_degenerate_axis_uses_vtk_denominator_fallback() {
        let pd = PolyData::from_points(vec![[2.0, 3.0, 0.0]]);
        let result = texture_map_to_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        let tc = result.point_data().tcoords().unwrap();
        let mut uv = [0.0f64; 2];
        tc.tuple_as_f64(0, &mut uv);
        assert_eq!(uv, [0.0, 6.0]);
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

    #[test]
    fn sphere_mapping_empty_input_does_not_allocate_tcoords() {
        let result = texture_map_to_sphere(&PolyData::new(), [0.0, 0.0, 0.0]);
        assert_eq!(result.points.len(), 0);
        assert!(result.point_data().tcoords().is_none());
    }
}
