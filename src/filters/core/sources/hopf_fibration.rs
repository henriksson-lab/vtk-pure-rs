//! Hopf fibration visualization (circles on S3 projected to R3).
use crate::data::{CellArray, Points, PolyData};
pub fn hopf_fibration(
    num_fibers: usize,
    fiber_resolution: usize,
    base_resolution: usize,
) -> PolyData {
    let nf = num_fibers.max(4);
    let fr = fiber_resolution.max(8);
    let br = base_resolution.max(nf);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Sample points on S2 (base space) and draw corresponding S1 fibers
    for fi in 0..nf {
        let theta = std::f64::consts::PI * fi as f64 / (nf - 1).max(1) as f64;
        let st = theta.sin();
        let ct = theta.cos();
        for phii in 0..br {
            let phi = 2.0 * std::f64::consts::PI * phii as f64 / br as f64;
            // Point on S2: (st*cos(phi), st*sin(phi), ct)
            let b = [st * phi.cos(), st * phi.sin(), ct];
            // Hopf fiber: circle parameterized by t
            let mut fiber_ids = Vec::new();
            for ti in 0..=fr {
                let t = 2.0 * std::f64::consts::PI * ti as f64 / fr as f64;
                // Stereographic projection of fiber from S3 to R3
                let ct2 = t.cos();
                let st2 = t.sin();
                let w = ct2;
                let denom = 1.0 / (1.0 - w * 0.5).max(0.1);
                let x = (b[0] * ct2 - b[1] * st2) * denom;
                let y = (b[0] * st2 + b[1] * ct2) * denom;
                let z = b[2] * denom;
                let idx = pts.len();
                pts.push([x, y, z]);
                fiber_ids.push(idx as i64);
            }
            if fiber_ids.len() >= 2 {
                lines.push_cell(&fiber_ids);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let h = hopf_fibration(3, 12, 4);
        assert!(h.lines.num_cells() >= 3);
        assert!(h.points.len() > 30);
    }
}
