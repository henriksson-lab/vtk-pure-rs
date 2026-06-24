//! Castle crenellation (battlement) wall top.
use crate::data::{CellArray, Points, PolyData};
pub fn crenellation(
    length: f64,
    wall_height: f64,
    merlon_height: f64,
    merlon_width: f64,
    gap_width: f64,
    thickness: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let ht = thickness / 2.0;
    let pitch = merlon_width + gap_width;
    let n_merlons = (length / pitch).floor() as usize;
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
    // Wall base
    add_box(&mut pts, &mut polys, 0.0, -ht, 0.0, length, ht, wall_height);
    // Merlons
    for i in 0..n_merlons {
        let x = i as f64 * pitch;
        add_box(
            &mut pts,
            &mut polys,
            x,
            -ht,
            wall_height,
            x + merlon_width,
            ht,
            wall_height + merlon_height,
        );
    }
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
        let c = crenellation(10.0, 2.0, 0.5, 0.8, 0.5, 0.3);
        assert!(c.polys.num_cells() > 6);
    }
}
