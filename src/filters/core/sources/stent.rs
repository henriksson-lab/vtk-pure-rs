//! Vascular stent (wire mesh tube).
use crate::data::{CellArray, Points, PolyData};
pub fn vascular_stent(
    radius: f64,
    length: f64,
    struts_per_ring: usize,
    num_rings: usize,
) -> PolyData {
    let ns = struts_per_ring.max(4);
    let nr = num_rings.max(2);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let ring_spacing = length / (nr - 1) as f64;
    // Zigzag rings
    for ri in 0..nr {
        let z = ri as f64 * ring_spacing;
        let phase = if ri % 2 == 0 {
            0.0
        } else {
            std::f64::consts::PI / ns as f64
        };
        for si in 0..ns * 2 {
            let a = std::f64::consts::PI * si as f64 / ns as f64 + phase;
            let zz = z + if si % 2 == 0 {
                ring_spacing * 0.2
            } else {
                -ring_spacing * 0.2
            };
            pts.push([radius * a.cos(), radius * a.sin(), zz]);
        }
    }
    // Connect zigzag within each ring
    for ri in 0..nr {
        let base = ri * ns * 2;
        let mut ring_ids: Vec<i64> = (0..ns * 2).map(|i| (base + i) as i64).collect();
        ring_ids.push(base as i64); // close ring
        lines.push_cell(&ring_ids);
    }
    // Connect rings longitudinally
    for ri in 0..nr - 1 {
        let base0 = ri * ns * 2;
        let base1 = (ri + 1) * ns * 2;
        for si in 0..ns {
            lines.push_cell(&[(base0 + si * 2) as i64, (base1 + si * 2) as i64]);
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
        let s = vascular_stent(0.003, 0.02, 6, 4);
        assert!(s.lines.num_cells() > 10);
    }
}
