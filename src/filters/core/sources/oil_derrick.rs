//! Oil derrick (pump jack) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn oil_derrick(base_w: f64, tower_h: f64, beam_l: f64, horsehead_r: f64) -> PolyData {
    let hw = base_w / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base
    let bb = pts.len();
    pts.push([-hw, -hw, 0.0]);
    pts.push([hw, -hw, 0.0]);
    pts.push([hw, hw, 0.0]);
    pts.push([-hw, hw, 0.0]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // A-frame tower
    let spread = hw * 0.8;
    for &y in &[-hw * 0.3, hw * 0.3] {
        let tb = pts.len();
        pts.push([-spread, y, 0.0]);
        pts.push([spread, y, 0.0]);
        pts.push([0.0, y, tower_h]);
        lines.push_cell(&[tb as i64, (tb + 2) as i64]);
        lines.push_cell(&[(tb + 1) as i64, (tb + 2) as i64]);
    }
    // Crossbar at top
    let cb = pts.len();
    pts.push([0.0, -hw * 0.3, tower_h]);
    pts.push([0.0, hw * 0.3, tower_h]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    // Walking beam (pivots at tower top)
    let beam_back = beam_l * 0.35;
    let beam_front = beam_l * 0.65;
    let wb = pts.len();
    pts.push([-beam_back, 0.0, tower_h]);
    pts.push([beam_front, 0.0, tower_h]);
    lines.push_cell(&[wb as i64, (wb + 1) as i64]);
    // Horsehead (curved piece at front of beam)
    let steps = 6;
    let mut hh_ids = Vec::new();
    for i in 0..=steps {
        let a = std::f64::consts::PI * 0.5 * i as f64 / steps as f64;
        let idx = pts.len();
        pts.push([
            beam_front + horsehead_r * a.sin(),
            0.0,
            tower_h - horsehead_r * (1.0 - a.cos()),
        ]);
        hh_ids.push(idx as i64);
    }
    lines.push_cell(&hh_ids);
    // Polished rod (vertical from horsehead to ground)
    let prb = pts.len();
    pts.push([beam_front + horsehead_r, 0.0, tower_h - horsehead_r]);
    pts.push([beam_front + horsehead_r, 0.0, 0.0]);
    lines.push_cell(&[prb as i64, (prb + 1) as i64]);
    // Counterweight (box on back of beam)
    let cw = beam_l * 0.1;
    let cwb = pts.len();
    pts.push([-beam_back - cw, -cw, tower_h - cw * 2.0]);
    pts.push([-beam_back + cw, -cw, tower_h - cw * 2.0]);
    pts.push([-beam_back + cw, cw, tower_h - cw * 2.0]);
    pts.push([-beam_back - cw, cw, tower_h - cw * 2.0]);
    pts.push([-beam_back - cw, -cw, tower_h]);
    pts.push([-beam_back + cw, -cw, tower_h]);
    pts.push([-beam_back + cw, cw, tower_h]);
    pts.push([-beam_back - cw, cw, tower_h]);
    let f = |i: usize| (cwb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Pitman arm (connecting crank to beam)
    let pab = pts.len();
    pts.push([-beam_back * 0.5, 0.0, tower_h * 0.3]);
    pts.push([-beam_back, 0.0, tower_h]);
    lines.push_cell(&[pab as i64, (pab + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let d = oil_derrick(3.0, 5.0, 6.0, 1.0);
        assert!(d.polys.num_cells() > 5);
        assert!(d.lines.num_cells() > 8);
    }
}
