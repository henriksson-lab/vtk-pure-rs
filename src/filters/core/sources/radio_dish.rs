//! Parabolic radio dish/satellite dish geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn parabolic_dish(radius: f64, depth: f64, u_res: usize, v_res: usize) -> PolyData {
    let ur = u_res.max(3);
    let vr = v_res.max(2);
    let f = radius * radius / (4.0 * depth); // focal length
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=vr {
        let r = radius * iv as f64 / vr as f64;
        let z = r * r / (4.0 * f);
        for iu in 0..=ur {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    let w = ur + 1;
    for iv in 0..vr {
        for iu in 0..ur {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn dish_with_feed(radius: f64, depth: f64, u_res: usize, v_res: usize) -> PolyData {
    let mut dish = parabolic_dish(radius, depth, u_res, v_res);
    let f = radius * radius / (4.0 * depth);
    // Add feed support struts as lines
    let mut lines = CellArray::new();
    let feed_idx = dish.points.len();
    dish.points.push([0.0, 0.0, f]);
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
        let edge_idx = dish.points.len();
        dish.points
            .push([radius * 0.9 * a.cos(), radius * 0.9 * a.sin(), depth * 0.9]);
        lines.push_cell(&[feed_idx as i64, edge_idx as i64]);
    }
    dish.lines = lines;
    dish
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dish() {
        let d = parabolic_dish(3.0, 1.0, 16, 8);
        assert!(d.points.len() > 50);
    }
    #[test]
    fn test_feed() {
        let d = dish_with_feed(3.0, 1.0, 16, 8);
        assert!(d.lines.num_cells() == 3);
    }
}
