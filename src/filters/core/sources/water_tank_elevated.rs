//! Elevated water tank (spherical/cylindrical on tower).
use crate::data::{CellArray, Points, PolyData};
pub fn elevated_spherical_tank(
    tank_r: f64,
    tower_h: f64,
    num_legs: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nl = num_legs.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Sphere (two hemispheres)
    let _vres = res / 2;
    for iv in 0..=res {
        let v = std::f64::consts::PI * iv as f64 / res as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([
                tank_r * sv * u.cos(),
                tank_r * sv * u.sin(),
                tower_h + tank_r * cv,
            ]);
        }
    }
    let w = res + 1;
    for iv in 0..res {
        for iu in 0..res {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Legs
    for li in 0..nl {
        let a = 2.0 * std::f64::consts::PI * li as f64 / nl as f64;
        let x = tank_r * 0.7 * a.cos();
        let y = tank_r * 0.7 * a.sin();
        let lb = pts.len();
        pts.push([x, y, 0.0]);
        pts.push([x, y, tower_h - tank_r]);
        lines.push_cell(&[lb as i64, (lb + 1) as i64]);
    }
    // Cross bracing
    for li in 0..nl {
        let li2 = (li + 1) % nl;
        let a1 = 2.0 * std::f64::consts::PI * li as f64 / nl as f64;
        let a2 = 2.0 * std::f64::consts::PI * li2 as f64 / nl as f64;
        let cb = pts.len();
        pts.push([
            tank_r * 0.7 * a1.cos(),
            tank_r * 0.7 * a1.sin(),
            tower_h * 0.4,
        ]);
        pts.push([
            tank_r * 0.7 * a2.cos(),
            tank_r * 0.7 * a2.sin(),
            tower_h * 0.4,
        ]);
        lines.push_cell(&[cb as i64, (cb + 1) as i64]);
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
        let t = elevated_spherical_tank(3.0, 15.0, 4, 10);
        assert!(t.polys.num_cells() > 50);
        assert!(t.lines.num_cells() >= 4);
    }
}
