//! Curved pergola with arched rafters.
use crate::data::{CellArray, Points, PolyData};
pub fn curved_pergola(
    width: f64,
    depth: f64,
    height: f64,
    arch_height: f64,
    num_arches: usize,
    resolution: usize,
) -> PolyData {
    let na = num_arches.max(2);
    let res = resolution.max(4);
    let pw = width * 0.04;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
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
    // Pillars
    for &y in &[0.0f64, depth] {
        for pi in 0..na {
            let x = pi as f64 * width / (na - 1) as f64;
            add_box(
                &mut pts,
                &mut polys,
                x - pw,
                y - pw,
                0.0,
                x + pw,
                y + pw,
                height,
            );
        }
    }
    // Arched rafters (along Y direction)
    for ai in 0..na {
        let x = ai as f64 * width / (na - 1) as f64;
        let mut arch_ids = Vec::new();
        for i in 0..=res {
            let t = i as f64 / res as f64;
            let y = t * depth;
            let z = height + arch_height * (1.0 - (2.0 * t - 1.0).powi(2));
            let idx = pts.len();
            pts.push([x, y, z]);
            arch_ids.push(idx as i64);
        }
        lines.push_cell(&arch_ids);
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
        let p = curved_pergola(4.0, 3.0, 2.5, 0.5, 3, 8);
        assert!(p.polys.num_cells() > 20);
        assert!(p.lines.num_cells() >= 3);
    }
}
