//! Steiner Roman surface (self-intersecting surface).
use crate::data::{CellArray, Points, PolyData};
pub fn roman_surface(scale: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=res {
        let v = std::f64::consts::PI * iv as f64 / res as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let su = u.sin();
            let cu = u.cos();
            let x = scale * sv * sv * su * cu;
            let y = scale * su * cv;
            let z = scale * cu * cv;
            pts.push([x, y, z]);
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
        let r = roman_surface(1.0, 12);
        assert!(r.polys.num_cells() > 100);
    }
}
