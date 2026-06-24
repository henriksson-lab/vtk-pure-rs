//! Picket fence geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn picket_fence(
    length: f64,
    height: f64,
    pickets: usize,
    picket_width: f64,
    rail_height: f64,
) -> PolyData {
    let np = pickets.max(2);
    let spacing = length / (np - 1) as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let pw = picket_width / 2.0;
    let t = picket_width * 0.3;
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
    // Pickets
    for i in 0..np {
        let x = i as f64 * spacing;
        add_box(&mut pts, &mut polys, x - pw, -t, 0.0, x + pw, t, height);
    }
    // Rails
    add_box(
        &mut pts,
        &mut polys,
        0.0,
        -t * 0.5,
        rail_height,
        length,
        t * 0.5,
        rail_height + t,
    );
    add_box(
        &mut pts,
        &mut polys,
        0.0,
        -t * 0.5,
        height * 0.7,
        length,
        t * 0.5,
        height * 0.7 + t,
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
        let f = picket_fence(5.0, 1.5, 6, 0.1, 0.3);
        assert!(f.polys.num_cells() > 30);
    }
}
