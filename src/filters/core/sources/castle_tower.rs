//! Castle tower with crenellations.
use crate::data::{CellArray, Points, PolyData};
pub fn castle_tower(
    radius: f64,
    height: f64,
    merlon_h: f64,
    num_merlons: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(num_merlons * 2);
    let nm = num_merlons.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Cylinder wall
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Floor
    let fc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[fc as i64, j as i64, i as i64]);
    }
    // Crenellations (alternate raised/lowered segments)
    let merlon_arc = 2.0 * std::f64::consts::PI / nm as f64;
    let gap_frac = 0.4;
    for mi in 0..nm {
        let a0 = mi as f64 * merlon_arc;
        let a1 = a0 + merlon_arc * (1.0 - gap_frac);
        let steps = 3;
        for si in 0..steps {
            let sa = a0 + si as f64 * (a1 - a0) / steps as f64;
            let sb = a0 + (si + 1) as f64 * (a1 - a0) / steps as f64;
            let b = pts.len();
            pts.push([radius * sa.cos(), radius * sa.sin(), height]);
            pts.push([radius * sb.cos(), radius * sb.sin(), height]);
            pts.push([radius * sb.cos(), radius * sb.sin(), height + merlon_h]);
            pts.push([radius * sa.cos(), radius * sa.sin(), height + merlon_h]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        }
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
        let t = castle_tower(3.0, 10.0, 1.0, 8, 16);
        assert!(t.polys.num_cells() > 20);
    }
}
