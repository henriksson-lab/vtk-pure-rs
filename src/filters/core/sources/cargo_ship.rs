//! Cargo ship geometry (hull + containers + bridge).
use crate::data::{CellArray, Points, PolyData};
pub fn cargo_ship(
    hull_l: f64,
    hull_w: f64,
    hull_h: f64,
    bridge_h: f64,
    num_container_rows: usize,
) -> PolyData {
    let hl = hull_l / 2.0;
    let hw = hull_w / 2.0;
    let ncr = num_container_rows.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    // Hull (simplified box with pointed bow)
    let hb = pts.len();
    pts.push([-hl * 1.1, 0.0, -hull_h]); // bow point
    pts.push([-hl * 0.7, -hw, -hull_h * 0.5]);
    pts.push([-hl * 0.7, hw, -hull_h * 0.5]);
    pts.push([hl, -hw, -hull_h * 0.5]);
    pts.push([hl, hw, -hull_h * 0.5]);
    pts.push([-hl * 0.7, -hw, 0.0]);
    pts.push([-hl * 0.7, hw, 0.0]);
    pts.push([hl, -hw, 0.0]);
    pts.push([hl, hw, 0.0]);
    // Hull sides
    polys.push_cell(&[(hb) as i64, (hb + 1) as i64, (hb + 5) as i64]); // bow port
    polys.push_cell(&[(hb) as i64, (hb + 6) as i64, (hb + 2) as i64]); // bow starboard
    polys.push_cell(&[
        (hb + 1) as i64,
        (hb + 3) as i64,
        (hb + 7) as i64,
        (hb + 5) as i64,
    ]); // port side
    polys.push_cell(&[
        (hb + 2) as i64,
        (hb + 6) as i64,
        (hb + 8) as i64,
        (hb + 4) as i64,
    ]); // starboard
    polys.push_cell(&[
        (hb + 3) as i64,
        (hb + 4) as i64,
        (hb + 8) as i64,
        (hb + 7) as i64,
    ]); // stern
        // Deck
    polys.push_cell(&[
        (hb + 5) as i64,
        (hb + 7) as i64,
        (hb + 8) as i64,
        (hb + 6) as i64,
    ]);
    // Containers (stacked boxes)
    let cont_l = hull_l * 0.06;
    let cont_w = hull_w * 0.35;
    let cont_h = hull_h * 0.4;
    for ri in 0..ncr {
        let x = -hl * 0.5 + ri as f64 * (hull_l * 0.7) / ncr as f64;
        for &y_offset in &[-cont_w * 0.55, cont_w * 0.55] {
            ab(
                &mut pts,
                &mut polys,
                x,
                y_offset - cont_w / 2.0,
                0.0,
                x + cont_l,
                y_offset + cont_w / 2.0,
                cont_h,
            );
            ab(
                &mut pts,
                &mut polys,
                x,
                y_offset - cont_w / 2.0,
                cont_h,
                x + cont_l,
                y_offset + cont_w / 2.0,
                cont_h * 2.0,
            );
        }
    }
    // Bridge (superstructure at stern)
    let bridge_x = hl * 0.6;
    let bridge_w = hull_w * 0.5;
    ab(
        &mut pts,
        &mut polys,
        bridge_x,
        -bridge_w / 2.0,
        0.0,
        hl,
        bridge_w / 2.0,
        bridge_h,
    );
    // Funnel
    ab(
        &mut pts,
        &mut polys,
        bridge_x + hull_l * 0.05,
        -hull_w * 0.08,
        bridge_h,
        bridge_x + hull_l * 0.08,
        hull_w * 0.08,
        bridge_h + hull_h * 0.6,
    );
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let s = cargo_ship(50.0, 8.0, 3.0, 6.0, 5);
        assert!(s.polys.num_cells() > 30);
    }
}
