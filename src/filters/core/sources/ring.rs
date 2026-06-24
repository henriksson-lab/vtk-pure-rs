use crate::data::PolyData;

/// Parameters for generating a 3D ring (thick circle / tube bent into a circle).
pub struct RingParams {
    /// Major radius (center of ring). Default: 1.0
    pub radius: f64,
    /// Tube radius (cross section). Default: 0.1
    pub tube_radius: f64,
    /// Segments around the ring. Default: 32
    pub ring_resolution: usize,
    /// Segments around tube cross-section. Default: 12
    pub tube_resolution: usize,
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for RingParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            tube_radius: 0.1,
            ring_resolution: 32,
            tube_resolution: 12,
            center: [0.0; 3],
        }
    }
}

/// Generate a ring (same geometry as torus but explicitly named for clarity).
pub fn ring(params: &RingParams) -> PolyData {
    crate::filters::core::sources::torus::torus(
        &crate::filters::core::sources::torus::TorusParams {
            ring_radius: params.radius,
            cross_section_radius: params.tube_radius,
            ring_resolution: params.ring_resolution,
            cross_section_resolution: params.tube_resolution,
            center: params.center,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ring() {
        let pd = ring(&RingParams::default());
        assert!(pd.points.len() > 100);
        assert!(pd.polys.num_cells() > 100);
    }

    #[test]
    fn small_ring() {
        let pd = ring(&RingParams {
            ring_resolution: 4,
            tube_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
    }
}
