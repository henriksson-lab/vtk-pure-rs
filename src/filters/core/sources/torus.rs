use crate::data::PolyData;

/// Parameters for generating a torus.
pub struct TorusParams {
    /// Major radius (center of tube to center of torus). Default: 1.0
    pub ring_radius: f64,
    /// Minor radius (radius of the tube). Default: 0.5
    pub cross_section_radius: f64,
    /// Number of segments around the ring. Default: 32
    pub ring_resolution: usize,
    /// Number of segments around the cross-section. Default: 16
    pub cross_section_resolution: usize,
    /// Center of the torus. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for TorusParams {
    fn default() -> Self {
        Self {
            ring_radius: 1.0,
            cross_section_radius: 0.5,
            ring_resolution: 32,
            cross_section_resolution: 16,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a torus in the XY plane as PolyData with smooth normals.
///
/// Thin wrapper around [`crate::filters::core::sources::parametric::torus`],
/// the single implementation of `vtkParametricTorus` +
/// `vtkParametricFunctionSource` (u, v in [0, 2*PI], `JoinU = JoinV = 1`,
/// anti-clockwise ordering, triangulated output with normals and texture
/// coordinates). The result is translated to `center`, which VTK's parametric
/// function has no equivalent of and which leaves the normals unchanged.
pub fn torus(params: &TorusParams) -> PolyData {
    let mut pd = crate::filters::core::sources::parametric::torus_uv(
        params.ring_radius,
        params.cross_section_radius,
        params.ring_resolution,
        params.cross_section_resolution,
    );
    let [cx, cy, cz] = params.center;
    if cx != 0.0 || cy != 0.0 || cz != 0.0 {
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            pd.points.set(i, [cx + p[0], cy + p[1], cz + p[2]]);
        }
    }
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_torus() {
        let pd = torus(&TorusParams::default());
        // vtkParametricFunctionSource samples UResolution x VResolution points.
        assert_eq!(pd.points.len(), 32 * 16);
        // JoinU = JoinV = 1 -> PtsU * PtsV quads, two triangles each.
        assert_eq!(pd.polys.num_cells(), 32 * 16 * 2);
    }

    #[test]
    fn small_torus() {
        let pd = torus(&TorusParams {
            ring_resolution: 4,
            cross_section_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 4 * 3 * 2);
    }

    #[test]
    fn has_normals() {
        let pd = torus(&TorusParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
    }

    #[test]
    fn custom_center() {
        let pd = torus(&TorusParams {
            center: [10.0, 20.0, 30.0],
            ring_resolution: 4,
            cross_section_resolution: 3,
            ..Default::default()
        });
        // All points should be near the center
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            assert!((p[0] - 10.0).abs() < 2.0);
            assert!((p[1] - 20.0).abs() < 2.0);
            assert!((p[2] - 30.0).abs() < 1.0);
        }
    }
}
