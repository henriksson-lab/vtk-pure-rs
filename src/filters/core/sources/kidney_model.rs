//! Simplified kidney model (bean shape).
use crate::data::{CellArray, Points, PolyData};
pub fn kidney(size: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let s = size;
    let vres = res;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            // Bean shape: indented on one side
            let indent = 0.3 * (std::f64::consts::PI * 0.5 - u).cos().max(0.0);
            let r = s * (1.0 - indent * sv) * 0.4;
            let x = r * sv * u.cos();
            let y = r * sv * u.sin();
            let z = s * 0.6 * cv;
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
    // Ureter (tube from bottom)
    let ub = pts.len();
    let ur = s * 0.05;
    for ring in 0..=1 {
        let z = -s * 0.55 + if ring == 0 { 0.0 } else { -s * 0.4 };
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([s * 0.1 + ur * a.cos(), ur * a.sin(), z]);
        }
    }
    let hr = res / 2;
    for i in 0..hr {
        let j = (i + 1) % hr;
        polys.push_cell(&[
            (ub + i) as i64,
            (ub + j) as i64,
            (ub + hr + j) as i64,
            (ub + hr + i) as i64,
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
        let k = kidney(3.0, 8);
        assert!(k.polys.num_cells() > 40);
    }
}
