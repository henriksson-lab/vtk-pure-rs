//! Helipad (circle with H marking) as flat geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn helipad(radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Circular platform
    let center = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for i in 0..res {
        let j = if i + 1 < res { i + 2 } else { 1 };
        polys.push_cell(&[center as i64, (i + 1) as i64, j as i64]);
    }
    // H marking (lines as thin quads)
    let s = radius * 0.4;
    let w = radius * 0.05;
    let add_bar =
        |pts: &mut Points<f64>, polys: &mut CellArray, x0: f64, y0: f64, x1: f64, y1: f64| {
            let dx = x1 - x0;
            let dy = y1 - y0;
            let l = (dx * dx + dy * dy).sqrt().max(1e-15);
            let nx = -dy / l * w;
            let ny = dx / l * w;
            let b = pts.len();
            pts.push([x0 + nx, y0 + ny, 0.01]);
            pts.push([x1 + nx, y1 + ny, 0.01]);
            pts.push([x1 - nx, y1 - ny, 0.01]);
            pts.push([x0 - nx, y0 - ny, 0.01]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        };
    add_bar(&mut pts, &mut polys, -s * 0.5, -s, -s * 0.5, s); // left vertical
    add_bar(&mut pts, &mut polys, s * 0.5, -s, s * 0.5, s); // right vertical
    add_bar(&mut pts, &mut polys, -s * 0.5, 0.0, s * 0.5, 0.0); // horizontal bar
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
        let h = helipad(5.0, 24);
        assert!(h.polys.num_cells() > 20);
    }
}
