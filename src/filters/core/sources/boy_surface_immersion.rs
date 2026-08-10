//! Boy's surface (immersion of projective plane in 3D).
use super::boy_surface::{boy_surface as boy_surface_params, BoySurfaceParams};
use crate::data::PolyData;

/// Boy's surface centered at the origin, scaled uniformly by `scale`.
///
/// Scale/resolution convenience form of
/// [`crate::filters::core::sources::boy_surface::boy_surface`], which holds the
/// single `vtkParametricBoy` implementation. The resolution is clamped to a
/// minimum of 8.
pub fn boy_surface(scale: f64, resolution: usize) -> PolyData {
    boy_surface_params(&BoySurfaceParams {
        center: [0.0, 0.0, 0.0],
        radius: scale,
        resolution: resolution.max(8),
        z_scale: 0.125,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let b = boy_surface(1.0, 12);
        assert!(b.polys.num_cells() > 100);
    }
}
