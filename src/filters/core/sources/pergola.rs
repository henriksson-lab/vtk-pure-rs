//! Pergola (open-roof garden structure) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn pergola(
    width: f64,
    depth: f64,
    height: f64,
    num_pillars_w: usize,
    num_rafters: usize,
) -> PolyData {
    let npw = num_pillars_w.max(2);
    let nr = num_rafters.max(2);
    let pw = width * 0.04;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let _lines = CellArray::new();
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
    // Pillars (two rows)
    for side_y in [0.0f64, depth] {
        for pi in 0..npw {
            let x = pi as f64 * width / (npw - 1) as f64;
            add_box(
                &mut pts,
                &mut polys,
                x - pw,
                side_y - pw,
                0.0,
                x + pw,
                side_y + pw,
                height,
            );
        }
    }
    // Beams (along Y at top)
    for pi in 0..npw {
        let x = pi as f64 * width / (npw - 1) as f64;
        add_box(
            &mut pts,
            &mut polys,
            x - pw,
            -pw,
            height,
            x + pw,
            depth + pw,
            height + pw * 2.0,
        );
    }
    // Rafters (along X on top of beams)
    for ri in 0..nr {
        let y = ri as f64 * depth / (nr - 1) as f64;
        add_box(
            &mut pts,
            &mut polys,
            -pw * 2.0,
            y - pw * 0.5,
            height + pw * 2.0,
            width + pw * 2.0,
            y + pw * 0.5,
            height + pw * 3.0,
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
        let p = pergola(4.0, 3.0, 2.5, 3, 5);
        assert!(p.polys.num_cells() > 30);
    }
}
