//! Cardiac pacemaker (generator + lead wires).
use crate::data::{CellArray, Points, PolyData};
pub fn pacemaker(
    body_w: f64,
    body_h: f64,
    body_t: f64,
    lead_length: f64,
    num_leads: usize,
    resolution: usize,
) -> PolyData {
    let _res = resolution.max(6);
    let nl = num_leads.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Generator body (rounded box)
    let hw = body_w / 2.0;
    let hh = body_h / 2.0;
    let ht = body_t / 2.0;
    let gb = pts.len();
    pts.push([-hw, -hh, -ht]);
    pts.push([hw, -hh, -ht]);
    pts.push([hw, hh, -ht]);
    pts.push([-hw, hh, -ht]);
    pts.push([-hw, -hh, ht]);
    pts.push([hw, -hh, ht]);
    pts.push([hw, hh, ht]);
    pts.push([-hw, hh, ht]);
    let f = |i: usize| (gb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Header (connector block)
    let header_w = body_w * 0.4;
    let header_h = body_h * 0.15;
    let hb = pts.len();
    pts.push([-header_w / 2.0, hh, -(body_t * 0.3)]);
    pts.push([header_w / 2.0, hh, -(body_t * 0.3)]);
    pts.push([header_w / 2.0, hh + header_h, -(body_t * 0.3)]);
    pts.push([-header_w / 2.0, hh + header_h, -(body_t * 0.3)]);
    pts.push([-header_w / 2.0, hh, (body_t * 0.3)]);
    pts.push([header_w / 2.0, hh, (body_t * 0.3)]);
    pts.push([header_w / 2.0, hh + header_h, (body_t * 0.3)]);
    pts.push([-header_w / 2.0, hh + header_h, (body_t * 0.3)]);
    let g = |i: usize| (hb + i) as i64;
    polys.push_cell(&[g(4), g(5), g(6), g(7)]);
    polys.push_cell(&[g(0), g(1), g(5), g(4)]);
    // Lead wires (curved lines from header to heart)
    for li in 0..nl {
        let lx = (li as f64 - nl as f64 / 2.0 + 0.5) * header_w / (nl as f64);
        let lead_steps = 16;
        let mut lead_ids = Vec::new();
        for si in 0..=lead_steps {
            let t = si as f64 / lead_steps as f64;
            let x = lx;
            let y = hh + header_h + t * lead_length;
            let z = (t * std::f64::consts::PI * 2.0).sin() * lead_length * 0.1;
            let idx = pts.len();
            pts.push([x, y, z]);
            lead_ids.push(idx as i64);
        }
        lines.push_cell(&lead_ids);
        // Electrode tip (small sphere)
        let tip_y = hh + header_h + lead_length;
        let eb = pts.len();
        let er = body_w * 0.03;
        pts.push([lx + er, tip_y, 0.0]);
        pts.push([lx - er, tip_y, 0.0]);
        pts.push([lx, tip_y + er, 0.0]);
        pts.push([lx, tip_y - er, 0.0]);
        pts.push([lx, tip_y, er]);
        pts.push([lx, tip_y, -er]);
        let faces = [
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [0, 5, 2],
            [2, 5, 1],
            [1, 5, 3],
            [3, 5, 0],
        ];
        for face in &faces {
            polys.push_cell(&[
                (eb + face[0]) as i64,
                (eb + face[1]) as i64,
                (eb + face[2]) as i64,
            ]);
        }
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
        let p = pacemaker(0.05, 0.04, 0.008, 0.15, 2, 6);
        assert!(p.polys.num_cells() > 15);
        assert!(p.lines.num_cells() >= 2);
    }
}
