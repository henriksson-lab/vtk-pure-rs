//! Victorian house with bay window and porch.
use crate::data::{CellArray, Points, PolyData};
pub fn victorian_house(width: f64, depth: f64, wall_h: f64, roof_h: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
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
    // Main building
    add_box(&mut pts, &mut polys, -hw, -hd, 0.0, hw, hd, wall_h);
    // Gable roof
    let rb = pts.len();
    pts.push([-hw, -hd, wall_h]);
    pts.push([hw, -hd, wall_h]);
    pts.push([hw, hd, wall_h]);
    pts.push([-hw, hd, wall_h]);
    pts.push([0.0, -hd, wall_h + roof_h]);
    pts.push([0.0, hd, wall_h + roof_h]);
    polys.push_cell(&[rb as i64, (rb + 4) as i64, (rb + 5) as i64, (rb + 3) as i64]); // left slope
    polys.push_cell(&[
        (rb + 1) as i64,
        (rb + 2) as i64,
        (rb + 5) as i64,
        (rb + 4) as i64,
    ]); // right slope
    polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 4) as i64]); // front gable
    polys.push_cell(&[(rb + 2) as i64, (rb + 3) as i64, (rb + 5) as i64]); // back gable
                                                                           // Bay window (protruding box on front)
    let bw = width * 0.25;
    let bd = depth * 0.15;
    add_box(
        &mut pts,
        &mut polys,
        -bw,
        -hd - bd,
        0.0,
        bw,
        -hd,
        wall_h * 0.7,
    );
    // Porch (flat roof at front)
    let pw = width * 0.4;
    let pd = depth * 0.2;
    let ph = wall_h * 0.4;
    add_box(
        &mut pts,
        &mut polys,
        -pw,
        -hd - pd,
        ph,
        pw,
        -hd,
        ph + width * 0.02,
    );
    // Porch pillars
    for &x in &[-pw + width * 0.03, pw - width * 0.03] {
        add_box(
            &mut pts,
            &mut polys,
            x - width * 0.02,
            -hd - pd + depth * 0.01,
            0.0,
            x + width * 0.02,
            -hd - pd + depth * 0.03,
            ph,
        );
    }
    // Chimney
    let cw = width * 0.08;
    add_box(
        &mut pts,
        &mut polys,
        hw * 0.4,
        -hd * 0.2,
        wall_h,
        hw * 0.4 + cw,
        hd * 0.2,
        wall_h + roof_h * 1.2,
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
        let h = victorian_house(8.0, 10.0, 5.0, 3.0);
        assert!(h.polys.num_cells() > 30);
    }
}
