//! Steiner Roman surface (self-intersecting surface).
use crate::data::{CellArray, Points, PolyData};
pub fn roman_surface(scale: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let radius_squared = scale * scale;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iu in 0..=res {
        let u = std::f64::consts::PI * iu as f64 / res as f64;
        let su = u.sin();
        let cu = u.cos();
        let s2u = (2.0 * u).sin();
        for iv in 0..=res {
            let v = std::f64::consts::PI * iv as f64 / res as f64;
            let cv = v.cos();
            let cv2 = cv * cv;
            let s2v = (2.0 * v).sin();
            let x = radius_squared * cv2 * s2u / 2.0;
            let y = radius_squared * su * s2v / 2.0;
            let z = radius_squared * cu * s2v / 2.0;
            pts.push([x, y, z]);
        }
    }
    let w = res + 1;
    for iu in 0..res {
        for iv in 0..res {
            polys.push_cell(&[
                (iu * w + iv) as i64,
                (iu * w + iv + 1) as i64,
                ((iu + 1) * w + iv + 1) as i64,
                ((iu + 1) * w + iv) as i64,
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
