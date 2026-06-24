//! Table geometry (top + 4 legs).
use crate::data::{CellArray, Points, PolyData};
pub fn table(top_w: f64, top_d: f64, top_h: f64, leg_w: f64, height: f64) -> PolyData {
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
    // Table top
    add_box(
        &mut pts,
        &mut polys,
        0.0,
        0.0,
        height - top_h,
        top_w,
        top_d,
        height,
    );
    // 4 legs
    let lh = height - top_h;
    add_box(&mut pts, &mut polys, 0.0, 0.0, 0.0, leg_w, leg_w, lh);
    add_box(
        &mut pts,
        &mut polys,
        top_w - leg_w,
        0.0,
        0.0,
        top_w,
        leg_w,
        lh,
    );
    add_box(
        &mut pts,
        &mut polys,
        0.0,
        top_d - leg_w,
        0.0,
        leg_w,
        top_d,
        lh,
    );
    add_box(
        &mut pts,
        &mut polys,
        top_w - leg_w,
        top_d - leg_w,
        0.0,
        top_w,
        top_d,
        lh,
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
        let t = table(2.0, 1.0, 0.05, 0.1, 0.8);
        assert_eq!(t.polys.num_cells(), 30);
    }
}
