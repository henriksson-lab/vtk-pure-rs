//! Beveled cylinder with chamfered edges.
use crate::data::{CellArray, Points, PolyData};
pub fn cylinder_beveled(radius: f64, height: f64, bevel: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let hh = height / 2.0;
    let b = bevel.max(0.0).min(hh).min(radius);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // 4 rings: bottom bevel, bottom main, top main, top bevel
    let rings: Vec<(f64, f64)> = vec![
        (radius - b, -hh),
        (radius, -hh + b),
        (radius, hh - b),
        (radius - b, hh),
    ];
    for &(r, z) in &rings {
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for ri in 0..rings.len() - 1 {
        for i in 0..res {
            let j = (i + 1) % res;
            let r0 = ri * res;
            let r1 = (ri + 1) * res;
            polys.push_cell(&[
                (r0 + i) as i64,
                (r0 + j) as i64,
                (r1 + j) as i64,
                (r1 + i) as i64,
            ]);
        }
    }
    // Bottom cap
    let bc = pts.len();
    pts.push([0.0, 0.0, -hh]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bc as i64, j as i64, i as i64]);
    }
    // Top cap
    let tc = pts.len();
    pts.push([0.0, 0.0, hh]);
    let top = (rings.len() - 1) * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[tc as i64, (top + i) as i64, (top + j) as i64]);
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
        let c = cylinder_beveled(1.0, 2.0, 0.2, 12);
        assert!(c.points.len() > 40);
        assert!(c.polys.num_cells() > 30);
    }
}
