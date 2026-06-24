use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a star polygon.
pub struct StarParams {
    /// Number of points on the star. Default: 5
    pub num_points: usize,
    /// Outer radius. Default: 1.0
    pub outer_radius: f64,
    /// Inner radius (valley between tips). Default: 0.4
    pub inner_radius: f64,
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for StarParams {
    fn default() -> Self {
        Self {
            num_points: 5,
            outer_radius: 1.0,
            inner_radius: 0.4,
            center: [0.0; 3],
        }
    }
}

/// Generate a star polygon in the XY plane as a filled PolyData.
pub fn star(params: &StarParams) -> PolyData {
    let n = params.num_points.max(3);
    let mut points = Points::new();
    let mut polys = CellArray::new();

    // Center point
    points.push(params.center);
    let n_verts = 2 * n;

    for i in 0..n_verts {
        let angle = PI * 2.0 * i as f64 / n_verts as f64 - PI / 2.0;
        let r = if i % 2 == 0 {
            params.outer_radius
        } else {
            params.inner_radius
        };
        points.push([
            params.center[0] + r * angle.cos(),
            params.center[1] + r * angle.sin(),
            params.center[2],
        ]);
    }

    // Fan triangulation from center
    for i in 0..n_verts {
        let a = (i + 1) as i64;
        let b = if i + 1 < n_verts { (i + 2) as i64 } else { 1 };
        polys.push_cell(&[0, a, b]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_star() {
        let pd = star(&StarParams::default());
        assert_eq!(pd.points.len(), 11); // center + 10 vertices
        assert_eq!(pd.polys.num_cells(), 10);
    }

    #[test]
    fn triangle_star() {
        let pd = star(&StarParams {
            num_points: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 7); // center + 6
        assert_eq!(pd.polys.num_cells(), 6);
    }
}
