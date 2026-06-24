use crate::data::{CellArray, Points, PolyData};

/// Parameters for generating a frustum (truncated pyramid).
pub struct FrustumParams {
    /// Bottom face half-width. Default: 0.5
    pub bottom_radius: f64,
    /// Top face half-width. Default: 0.25
    pub top_radius: f64,
    /// Height along Z axis. Default: 1.0
    pub height: f64,
    /// Number of sides. Default: 4 (square frustum)
    pub resolution: usize,
    /// Center of the bottom face. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Whether to cap the top and bottom. Default: true
    pub capping: bool,
}

impl Default for FrustumParams {
    fn default() -> Self {
        Self {
            bottom_radius: 0.5,
            top_radius: 0.25,
            height: 1.0,
            resolution: 4,
            center: [0.0, 0.0, 0.0],
            capping: true,
        }
    }
}

/// Generate a frustum (truncated cone/pyramid) along the Z axis.
pub fn frustum(params: &FrustumParams) -> PolyData {
    let n = params.resolution.max(3);
    let pi2 = 2.0 * std::f64::consts::PI;
    let cx = params.center[0];
    let cy = params.center[1];
    let cz = params.center[2];

    let mut points = Points::new();
    let mut polys = CellArray::new();

    // Bottom ring
    for i in 0..n {
        let angle = pi2 * i as f64 / n as f64;
        points.push([
            cx + params.bottom_radius * angle.cos(),
            cy + params.bottom_radius * angle.sin(),
            cz,
        ]);
    }

    // Top ring
    for i in 0..n {
        let angle = pi2 * i as f64 / n as f64;
        points.push([
            cx + params.top_radius * angle.cos(),
            cy + params.top_radius * angle.sin(),
            cz + params.height,
        ]);
    }

    // Side quads
    for i in 0..n {
        let i_next = (i + 1) % n;
        polys.push_cell(&[i as i64, i_next as i64, (n + i_next) as i64, (n + i) as i64]);
    }

    // Caps
    if params.capping {
        // Bottom cap (reversed winding for outward normal)
        let bottom: Vec<i64> = (0..n as i64).rev().collect();
        polys.push_cell(&bottom);

        // Top cap
        let top: Vec<i64> = (n as i64..2 * n as i64).collect();
        polys.push_cell(&top);
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
    fn default_frustum() {
        let pd = frustum(&FrustumParams::default());
        // 4 sides: 8 points, 4 side quads + 2 caps = 6 polys
        assert_eq!(pd.points.len(), 8);
        assert_eq!(pd.polys.num_cells(), 6);
    }

    #[test]
    fn frustum_no_cap() {
        let pd = frustum(&FrustumParams {
            capping: false,
            resolution: 6,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 6); // just side quads
    }

    #[test]
    fn cone_frustum() {
        // Top radius = 0 makes a cone (degenerate top face)
        let pd = frustum(&FrustumParams {
            top_radius: 0.0,
            resolution: 8,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 16);
    }
}
