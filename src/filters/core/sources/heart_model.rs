//! Simplified anatomical heart model.
use crate::data::{CellArray, Points, PolyData};
pub fn heart_model(size: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let s = size;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Heart shape via parametric surface
    let vres = res;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            // Cardioid-like cross section
            let r = s * (1.0 - 0.3 * cv) * (1.0 + 0.15 * u.cos());
            let x = r * sv * u.cos();
            let y = r * sv * u.sin() * 0.9;
            let z = s * cv * 1.2;
            pts.push([x, y, z]);
        }
    }
    let w = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Aorta (tube from top)
    let ab = pts.len();
    let ar = s * 0.15;
    let al = s * 0.8;
    for ring in 0..=1 {
        let z = s * 1.1 + if ring == 0 { 0.0 } else { al };
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([s * 0.2 + ar * a.cos(), ar * a.sin(), z]);
        }
    }
    let hr = res / 2;
    for i in 0..hr {
        let j = (i + 1) % hr;
        polys.push_cell(&[
            (ab + i) as i64,
            (ab + j) as i64,
            (ab + hr + j) as i64,
            (ab + hr + i) as i64,
        ]);
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
        let h = heart_model(3.0, 10);
        assert!(h.polys.num_cells() > 50);
    }
}
