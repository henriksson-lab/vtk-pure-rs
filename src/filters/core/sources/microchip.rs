//! Microchip/IC package geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn microchip(
    body_w: f64,
    body_l: f64,
    body_h: f64,
    num_pins: usize,
    pin_spacing: f64,
    pin_length: f64,
) -> PolyData {
    let np = num_pins.max(4);
    let hw = body_w / 2.0;
    let hl = body_l / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
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
    // Package body
    ab(&mut pts, &mut polys, -hw, -hl, 0.0, hw, hl, body_h);
    // Pin 1 marker (dot)
    let mc = pts.len();
    pts.push([-hw * 0.7, -hl * 0.7, body_h + 0.001]);
    let mr = body_w * 0.05;
    for i in 0..6 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 6.0;
        pts.push([
            -hw * 0.7 + mr * a.cos(),
            -hl * 0.7 + mr * a.sin(),
            body_h + 0.001,
        ]);
    }
    for i in 0..6 {
        let j = if i + 1 < 6 { mc + 2 + i } else { mc + 1 };
        polys.push_cell(&[mc as i64, (mc + 1 + i) as i64, j as i64]);
    }
    // Pins (DIP-style, two rows)
    let pins_per_side = np / 2;
    let pw = pin_spacing * 0.3;
    let ph = body_h * 0.15;
    for side in 0..2 {
        let y = if side == 0 { -hl } else { hl };
        let dy = if side == 0 { -pin_length } else { pin_length };
        for pi in 0..pins_per_side {
            let x = -hw * 0.8 + pi as f64 * pin_spacing;
            // Pin body (L-shaped: horizontal + vertical)
            let pb = pts.len();
            pts.push([x - pw, y, body_h * 0.3]);
            pts.push([x + pw, y, body_h * 0.3]);
            pts.push([x + pw, y + dy, body_h * 0.3]);
            pts.push([x - pw, y + dy, body_h * 0.3]);
            pts.push([x - pw, y, body_h * 0.3 + ph]);
            pts.push([x + pw, y, body_h * 0.3 + ph]);
            pts.push([x + pw, y + dy, 0.0]);
            pts.push([x - pw, y + dy, 0.0]);
            polys.push_cell(&[
                (pb) as i64,
                (pb + 1) as i64,
                (pb + 5) as i64,
                (pb + 4) as i64,
            ]); // side
            polys.push_cell(&[
                (pb + 1) as i64,
                (pb + 2) as i64,
                (pb + 6) as i64,
                (pb + 5) as i64,
            ]); // bottom bend
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
        let c = microchip(0.02, 0.03, 0.005, 16, 0.00254, 0.005);
        assert!(c.polys.num_cells() > 10);
    }
}
