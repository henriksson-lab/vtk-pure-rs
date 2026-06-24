//! Boy's surface (immersion of projective plane in 3D).
use crate::data::{CellArray, Points, PolyData};
pub fn boy_surface(scale: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=res {
        let v = std::f64::consts::FRAC_PI_2 * iv as f64 / res as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let sqrt2 = std::f64::consts::SQRT_2;
            let denom = 2.0 - sqrt2 * (3.0 * u).sin() * v.sin().powi(2) * 2.0f64.max(0.5);
            let x = scale * (cv * cv * (u).cos() + (sv * sv * 2.0 * u).cos() * (1.0 / sqrt2))
                / denom.max(0.1);
            let y = scale * (cv * cv * (u).sin() - (sv * sv * 2.0 * u).sin() * (1.0 / sqrt2))
                / denom.max(0.1);
            let z = scale * 3.0 * cv * sv / denom.max(0.1);
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
        let b = boy_surface(1.0, 12);
        assert!(b.polys.num_cells() > 100);
    }
}
