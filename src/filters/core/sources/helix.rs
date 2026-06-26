use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a helix (spiral) polyline.
pub struct HelixParams {
    /// Radius of helix. Default: 1.0
    pub radius: f64,
    /// Pitch of helix. Default: 1.0
    pub pitch: f64,
    /// Number of turns in helix. Default: 1
    pub number_of_turns: usize,
    /// Number of points per turn in helix. Default: 10
    pub resolution_per_turn: usize,
}

impl Default for HelixParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            pitch: 1.0,
            number_of_turns: 1,
            resolution_per_turn: 10,
        }
    }
}

/// Generate a helix as a polyline in PolyData.
pub fn helix(params: &HelixParams) -> PolyData {
    let number_of_turns = params.number_of_turns.max(1);
    let resolution_per_turn = params.resolution_per_turn.max(2);
    let n = resolution_per_turn * number_of_turns;

    let mut points = Points::new();
    let mut lines = CellArray::new();
    let mut cell_ids = Vec::with_capacity(n);
    let pi_twice = 2.0 * PI;
    let a = params.radius;
    let b = params.pitch / pi_twice;

    for i in 0..n {
        let t = number_of_turns as f64 * pi_twice * i as f64 / (n - 1) as f64;
        points.push([a * t.cos(), a * t.sin(), b * t]);
        cell_ids.push(i as i64);
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
        assert_eq!(pd.points.len(), 10);
        assert_eq!(pd.lines.num_cells(), 1);
    }

    #[test]
    fn start_and_end() {
        let pd = helix(&HelixParams {
            radius: 1.0,
            pitch: 5.0,
            number_of_turns: 1,
            resolution_per_turn: 8,
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
    fn multiple_turns() {
        let pd = helix(&HelixParams {
            pitch: 2.0,
            number_of_turns: 3,
            resolution_per_turn: 4,
            ..Default::default()
        });
        let last = pd.points.get(pd.points.len() - 1);
        assert_eq!(pd.points.len(), 12);
        assert!((last[2] - 6.0).abs() < 1e-10);
    }
}
