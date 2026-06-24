use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a helix (spiral) polyline.
pub struct HelixParams {
    /// Radius of the helix. Default: 1.0
    pub radius: f64,
    /// Number of complete turns. Default: 3.0
    pub turns: f64,
    /// Height (pitch * turns). Default: 2.0
    pub height: f64,
    /// Number of points per turn. Default: 32
    pub resolution: usize,
    /// Center of the helix base. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for HelixParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            turns: 3.0,
            height: 2.0,
            resolution: 32,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a helix as a polyline in PolyData.
///
/// The helix spirals around the Z axis, starting at the base center
/// and rising to `center[2] + height`.
pub fn helix(params: &HelixParams) -> PolyData {
    let res = params.resolution.max(4);
    let n_pts = (params.turns * res as f64).ceil() as usize + 1;
    let n_pts = n_pts.max(2);

    let mut points = Points::new();
    let mut lines = CellArray::new();

    let total_angle = 2.0 * PI * params.turns;
    let mut cell_ids: Vec<i64> = Vec::with_capacity(n_pts);

    for i in 0..n_pts {
        let t = i as f64 / (n_pts - 1) as f64;
        let angle = total_angle * t;
        let x = params.center[0] + params.radius * angle.cos();
        let y = params.center[1] + params.radius * angle.sin();
        let z = params.center[2] + params.height * t;

        let idx = points.len() as i64;
        points.push([x, y, z]);
        cell_ids.push(idx);
    }

    lines.push_cell(&cell_ids);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_helix() {
        let pd = helix(&HelixParams::default());
        // 3 turns * 32 pts/turn + 1 = 97
        assert_eq!(pd.points.len(), 97);
        assert_eq!(pd.lines.num_cells(), 1);
    }

    #[test]
    fn start_and_end() {
        let pd = helix(&HelixParams {
            radius: 1.0,
            turns: 1.0,
            height: 5.0,
            resolution: 8,
            center: [0.0, 0.0, 0.0],
            ..Default::default()
        });
        let first = pd.points.get(0);
        let last = pd.points.get(pd.points.len() - 1);
        // Start at z=0, end at z=5
        assert!((first[2]).abs() < 1e-10);
        assert!((last[2] - 5.0).abs() < 1e-10);
        // Both at radius 1 from center
        let r0 = (first[0] * first[0] + first[1] * first[1]).sqrt();
        let r1 = (last[0] * last[0] + last[1] * last[1]).sqrt();
        assert!((r0 - 1.0).abs() < 1e-10);
        assert!((r1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn custom_center() {
        let pd = helix(&HelixParams {
            center: [10.0, 20.0, 30.0],
            turns: 1.0,
            resolution: 4,
            ..Default::default()
        });
        let first = pd.points.get(0);
        assert!((first[2] - 30.0).abs() < 1e-10);
    }
}
