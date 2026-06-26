//! Offshore oil rig/platform geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn oil_rig(
    deck_w: f64,
    deck_d: f64,
    deck_h: f64,
    leg_height: f64,
    num_legs: usize,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hw = deck_w / 2.0;
    let hd = deck_d / 2.0;
    // Deck (box)
    let b = pts.len();
    pts.push([-hw, -hd, leg_height]);
    pts.push([hw, -hd, leg_height]);
    pts.push([hw, hd, leg_height]);
    pts.push([-hw, hd, leg_height]);
    pts.push([-hw, -hd, leg_height + deck_h]);
    pts.push([hw, -hd, leg_height + deck_h]);
    pts.push([hw, hd, leg_height + deck_h]);
    pts.push([-hw, hd, leg_height + deck_h]);
    let f = |i: usize| (b + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Legs
    let nl = num_legs.max(4);
    for li in 0..nl {
        let t = li as f64 / (nl - 1).max(1) as f64;
        let x = -hw * 0.8 + deck_w * 0.8 * t;
        for &y in &[-hd * 0.8, hd * 0.8] {
            let lb = pts.len();
            pts.push([x, y, 0.0]);
            pts.push([x, y, leg_height]);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
        }
    }
    // Cross bracing
    for li in 0..nl - 1 {
        let x0 = -hw * 0.8 + deck_w * 0.8 * li as f64 / (nl - 1).max(1) as f64;
        let x1 = -hw * 0.8 + deck_w * 0.8 * (li + 1) as f64 / (nl - 1).max(1) as f64;
        for &y in &[-hd * 0.8, hd * 0.8] {
            let cb = pts.len();
            pts.push([x0, y, 0.0]);
            pts.push([x1, y, leg_height * 0.5]);
            lines.push_cell(&[cb as i64, (cb + 1) as i64]);
        }
    }
    // Helipad (circle on top)
    let pad_r = deck_w.min(deck_d) * 0.2;
    let pad_z = leg_height + deck_h + 0.01;
    let pr = 16;
    let pc = pts.len();
    pts.push([0.0, 0.0, pad_z]);
    for i in 0..pr {
        let a = 2.0 * std::f64::consts::PI * i as f64 / pr as f64;
        pts.push([pad_r * a.cos(), pad_r * a.sin(), pad_z]);
    }
    for i in 0..pr {
        let j = if i + 1 < pr { pc + 2 + i } else { pc + 1 };
        polys.push_cell(&[pc as i64, (pc + 1 + i) as i64, j as i64]);
    }
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
        let p = oil_rig(20.0, 15.0, 3.0, 25.0, 4);
        assert!(p.polys.num_cells() > 10);
        assert!(p.lines.num_cells() > 5);
    }
}
