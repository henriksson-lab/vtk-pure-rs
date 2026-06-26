use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a Möbius strip.
pub struct MobiusParams {
    /// Radius of the center circle. Default: 1.0
    pub radius: f64,
    /// Half-width of the strip. Default: 0.3
    pub width: f64,
    /// Number of segments around the loop. Default: 64
    pub resolution: usize,
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for MobiusParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            width: 0.3,
            resolution: 64,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a Möbius strip as PolyData.
pub fn mobius(params: &MobiusParams) -> PolyData {
    let pts_u = params.resolution.max(8);
    let pts_v = 3;
    let radius = params.radius;
    let half_width = params.width;
    let [cx, cy, cz] = params.center;

    let mut points = Points::new();
    let mut polys = CellArray::new();

    for i in 0..pts_u {
        let u = 2.0 * PI * i as f64 / (pts_u - 1) as f64;
        for j in 0..pts_v {
            let v = -half_width + 2.0 * half_width * j as f64 / (pts_v - 1) as f64;
            let (pt, _, _) = evaluate_mobius(u, v, radius);
            points.push([cx + pt[0], cy + pt[1], cz + pt[2]]);
        }
    }

    for i in 0..(pts_u - 1) {
        for j in 0..(pts_v - 1) {
            let id1 = (j + i * pts_v) as i64;
            let id2 = id1 + pts_v as i64;
            let id3 = id2 + 1;
            let id4 = id1 + 1;
            polys.push_cell(&[id1, id2, id3]);
            polys.push_cell(&[id1, id3, id4]);
        }
    }
    for j in 0..(pts_v - 1) {
        let id1 = (j + (pts_u - 1) * pts_v) as i64;
        let id3 = id1 + 1;
        let id2 = (pts_v - 1 - j) as i64;
        let id4 = id2 - 1;
        polys.push_cell(&[id1, id2, id3]);
        polys.push_cell(&[id1, id3, id4]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// VTK `vtkParametricMobius::Evaluate`, translated with Rust names.
pub(crate) fn evaluate_mobius(u: f64, v: f64, radius: f64) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let cu = u.cos();
    let cu2 = (u / 2.0).cos();
    let su = u.sin();
    let su2 = (u / 2.0).sin();
    let t = radius - v * su2;

    let pt = [t * su, t * cu, v * cu2];

    let du = [
        -v * cu2 * su / 2.0 + pt[1],
        -v * cu2 * cu / 2.0 - pt[0],
        -v * su2 / 2.0,
    ];
    let dv = [-su2 * su, -su2 * cu, cu2];

    (pt, du, dv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mobius() {
        let pd = mobius(&MobiusParams::default());
        assert_eq!(pd.points.len(), 192);
        assert_eq!(pd.polys.num_cells(), 256);
    }

    #[test]
    fn small_mobius() {
        let pd = mobius(&MobiusParams {
            resolution: 8,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 24);
        assert_eq!(pd.polys.num_cells(), 32);
    }

    #[test]
    fn evaluate_matches_vtk_mobius() {
        let (pt, du, dv) = evaluate_mobius(0.0, 0.3, 1.0);
        assert_eq!(pt, [0.0, 1.0, 0.3]);
        assert_eq!(du, [1.0, -0.15, -0.0]);
        assert_eq!(dv, [-0.0, -0.0, 1.0]);
    }
}
