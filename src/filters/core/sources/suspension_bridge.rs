//! Suspension bridge geometry (towers, cables, deck).
use crate::data::{CellArray, Points, PolyData};
pub fn suspension_bridge(
    span: f64,
    tower_height: f64,
    deck_width: f64,
    cable_sag: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let hs = span / 2.0;
    let hw = deck_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Deck
    let d0 = pts.len();
    pts.push([-hs, -hw, 0.0]);
    pts.push([hs, -hw, 0.0]);
    pts.push([hs, hw, 0.0]);
    pts.push([-hs, hw, 0.0]);
    polys.push_cell(&[d0 as i64, (d0 + 1) as i64, (d0 + 2) as i64, (d0 + 3) as i64]);
    // Towers
    for tx in [-hs, hs] {
        let b = pts.len();
        pts.push([tx, -hw * 0.3, 0.0]);
        pts.push([tx, -hw * 0.3, tower_height]);
        pts.push([tx, hw * 0.3, 0.0]);
        pts.push([tx, hw * 0.3, tower_height]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        lines.push_cell(&[(b + 2) as i64, (b + 3) as i64]);
    }
    // Main cables (catenary approximation)
    for side_y in [-hw * 0.3, hw * 0.3] {
        let mut cable_ids = Vec::new();
        for i in 0..=res {
            let t = i as f64 / res as f64;
            let x = -hs + span * t;
            let sag_factor = 4.0 * t * (1.0 - t);
            let z = tower_height - cable_sag * sag_factor;
            let idx = pts.len();
            pts.push([x, side_y, z]);
            cable_ids.push(idx as i64);
        }
        lines.push_cell(&cable_ids);
    }
    // Suspender cables
    for side_y in [-hw * 0.3, hw * 0.3] {
        for i in 1..res {
            let t = i as f64 / res as f64;
            let x = -hs + span * t;
            let sag_factor = 4.0 * t * (1.0 - t);
            let z_top = tower_height - cable_sag * sag_factor;
            let b = pts.len();
            pts.push([x, side_y, 0.0]);
            pts.push([x, side_y, z_top]);
            lines.push_cell(&[b as i64, (b + 1) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let b = suspension_bridge(100.0, 30.0, 10.0, 15.0, 16);
        assert!(b.lines.num_cells() > 10);
    }
}
