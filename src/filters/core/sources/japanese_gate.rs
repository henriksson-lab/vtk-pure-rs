//! Torii gate (Japanese shrine gate) geometry.
use crate::data::{CellArray, Points, PolyData};

pub fn japanese_gate(width: f64, height: f64, beam_thickness: f64, pillar_radius: f64) -> PolyData {
    let hw = width / 2.0;
    let bt = beam_thickness;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let add_box = |pts: &mut Points<f64>,
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
    let pr = pillar_radius;
    // Pillars
    add_box(
        &mut pts,
        &mut polys,
        -hw - pr,
        -pr,
        0.0,
        -hw + pr,
        pr,
        height,
    );
    add_box(&mut pts, &mut polys, hw - pr, -pr, 0.0, hw + pr, pr, height);
    // Top beam (kasagi) - wider with slight curve
    let overhang = width * 0.15;
    add_box(
        &mut pts,
        &mut polys,
        -hw - overhang,
        -pr * 1.5,
        height,
        hw + overhang,
        pr * 1.5,
        height + bt,
    );
    // Lower beam (nuki)
    let nuki_h = height * 0.7;
    add_box(
        &mut pts,
        &mut polys,
        -hw,
        -pr * 0.8,
        nuki_h,
        hw,
        pr * 0.8,
        nuki_h + bt * 0.6,
    );
    // Roof-like top (curved kasagi approximation)
    let rb = pts.len();
    let rw = hw + overhang * 1.1;
    pts.push([-rw, -pr * 2.0, height + bt]);
    pts.push([rw, -pr * 2.0, height + bt]);
    pts.push([rw, pr * 2.0, height + bt]);
    pts.push([-rw, pr * 2.0, height + bt]);
    pts.push([0.0, 0.0, height + bt * 2.5]);
    polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 4) as i64]);
    polys.push_cell(&[(rb + 1) as i64, (rb + 2) as i64, (rb + 4) as i64]);
    polys.push_cell(&[(rb + 2) as i64, (rb + 3) as i64, (rb + 4) as i64]);
    polys.push_cell(&[(rb + 3) as i64, rb as i64, (rb + 4) as i64]);
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
        let t = japanese_gate(4.0, 5.0, 0.3, 0.2);
        assert!(t.polys.num_cells() > 20);
    }
}
