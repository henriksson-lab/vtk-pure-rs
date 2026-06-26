//! Boy's surface (immersion of projective plane in 3D).
use crate::data::{CellArray, Points, PolyData};
pub fn boy_surface(scale: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let z_scale = 0.125;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=res {
        let v = std::f64::consts::PI * iv as f64 / res as f64;
        for iu in 0..=res {
            let u = std::f64::consts::PI * iu as f64 / res as f64;
            let (point, _, _) = super::boy_surface::evaluate_boy(u, v, z_scale);
            pts.push([scale * point[0], scale * point[1], scale * point[2]]);
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
        let b = boy_surface(1.0, 12);
        assert!(b.polys.num_cells() > 100);
    }
}
