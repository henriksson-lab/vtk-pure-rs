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
    let n = params.resolution.max(8);
    let r = params.radius;
    let w = params.width;
    let steps = 2; // across the strip width

    let mut points = Points::new();
    let mut polys = CellArray::new();

    for i in 0..=n {
        let t = 2.0 * PI * i as f64 / n as f64;
        let half_twist = t / 2.0;

        let ct = t.cos();
        let st = t.sin();
        let ch = half_twist.cos();
        let sh = half_twist.sin();

        for j in 0..=steps {
            let s = -1.0 + 2.0 * j as f64 / steps as f64; // -1 to 1
            let x = params.center[0] + (r + w * s * ch) * ct;
            let y = params.center[1] + (r + w * s * ch) * st;
            let z = params.center[2] + w * s * sh;
            points.push([x, y, z]);
        }
    }

    let row = steps + 1;
    for i in 0..n {
        for j in 0..steps {
            let a = (i * row + j) as i64;
            let b = (i * row + j + 1) as i64;
            let c = ((i + 1) * row + j + 1) as i64;
            let d = ((i + 1) * row + j) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
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
    fn default_mobius() {
        let pd = mobius(&MobiusParams::default());
        assert!(pd.points.len() > 100);
        assert!(pd.polys.num_cells() > 50);
    }

    #[test]
    fn small_mobius() {
        let pd = mobius(&MobiusParams {
            resolution: 8,
            ..Default::default()
        });
        assert_eq!(pd.polys.num_cells(), 16); // 8 segments * 2 quads
    }
}
