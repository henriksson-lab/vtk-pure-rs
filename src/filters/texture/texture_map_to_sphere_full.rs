//! Full spherical texture mapping.
//!
//! This follows VTK's `vtkTextureMapToSphere` texture-coordinate generation.

use crate::data::PolyData;

use super::texture_map::texture_map_to_sphere_with_prevent_seam;

/// Generate spherical texture coordinates.
///
/// Uses the same point-wise coordinate formulas as VTK's
/// `vtkTextureMapToSphere`, including its `PreventSeam` option.
pub fn texture_map_to_sphere_full(
    input: &PolyData,
    center: [f64; 3],
    prevent_seam: bool,
) -> PolyData {
    texture_map_to_sphere_with_prevent_seam(input, center, prevent_seam)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sphere_points() -> PolyData {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        let n = 8;
        for i in 0..=n {
            let theta = std::f64::consts::PI * i as f64 / n as f64;
            for j in 0..=n {
                let phi = 2.0 * std::f64::consts::PI * j as f64 / n as f64 - std::f64::consts::PI;
                pts.push([
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ]);
            }
        }
        for i in 0..n {
            for j in 0..n {
                let bl = i * (n + 1) + j;
                tris.push([bl, bl + 1, bl + n + 2]);
                tris.push([bl, bl + n + 2, bl + n + 1]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn simple_mapping() {
        let mesh = make_sphere_points();
        let result = texture_map_to_sphere_full(&mesh, [0.0, 0.0, 0.0], false);
        let tc = result.point_data().tcoords().unwrap();
        assert_eq!(tc.num_tuples(), mesh.points.len());
        let mut buf = [0.0f64; 2];
        for i in 0..tc.num_tuples() {
            tc.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0 && buf[0] <= 1.0, "u={}", buf[0]);
            assert!(buf[1] >= 0.0 && buf[1] <= 1.0, "v={}", buf[1]);
        }
    }

    #[test]
    fn prevent_seam_mapping() {
        let mesh = make_sphere_points();
        let result = texture_map_to_sphere_full(&mesh, [0.0, 0.0, 0.0], true);
        assert_eq!(result.points.len(), mesh.points.len());
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn empty() {
        let result = texture_map_to_sphere_full(&PolyData::new(), [0.0; 3], true);
        assert_eq!(result.points.len(), 0);
    }
}
