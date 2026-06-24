//! Simplified eye model (cornea + iris + lens + retina).
use crate::data::{CellArray, Points, PolyData};
pub fn eye_model(radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let r = radius;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Sclera (main eyeball sphere, ~5/6 of sphere)
    let vres = res;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        if v < std::f64::consts::PI * 0.15 {
            continue;
        } // skip front pole for cornea
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([r * sv * u.cos(), r * sv * u.sin(), r * cv]);
        }
    }
    let w = res + 1;
    let rows = vres - (vres / 7);
    for iv in 0..rows {
        for iu in 0..res {
            if iv * w + iu + 1 < pts.len() && (iv + 1) * w + iu + 1 < pts.len() {
                polys.push_cell(&[
                    (iv * w + iu) as i64,
                    (iv * w + iu + 1) as i64,
                    ((iv + 1) * w + iu + 1) as i64,
                    ((iv + 1) * w + iu) as i64,
                ]);
            }
        }
    }
    // Cornea (slightly bulging front, smaller radius)
    let cr = r * 1.05;
    let cb = pts.len();
    let corn_res = res / 2;
    for iv in 0..=corn_res {
        let v = std::f64::consts::PI * 0.15 * iv as f64 / corn_res as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([cr * sv * u.cos(), cr * sv * u.sin(), cr * cv]);
        }
    }
    let cw = res + 1;
    for iv in 0..corn_res {
        for iu in 0..res {
            polys.push_cell(&[
                (cb + iv * cw + iu) as i64,
                (cb + iv * cw + iu + 1) as i64,
                (cb + (iv + 1) * cw + iu + 1) as i64,
                (cb + (iv + 1) * cw + iu) as i64,
            ]);
        }
    }
    // Pupil (dark circle at front)
    let pupil_r = r * 0.15;
    let pc = pts.len();
    pts.push([0.0, 0.0, r * 0.99]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([pupil_r * a.cos(), pupil_r * a.sin(), r * 0.98]);
    }
    for i in 0..res {
        let j = if i + 1 < res { pc + 2 + i } else { pc + 1 };
        polys.push_cell(&[pc as i64, (pc + 1 + i) as i64, j as i64]);
    }
    // Iris (ring around pupil)
    let iris_r = r * 0.35;
    let ib = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([iris_r * a.cos(), iris_r * a.sin(), r * 0.95]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (pc + 1 + i) as i64,
            (pc + 1 + j) as i64,
            (ib + j) as i64,
            (ib + i) as i64,
        ]);
    }
    // Optic nerve (small cylinder at back)
    let onb = pts.len();
    let onr = r * 0.08;
    for ring in 0..=1 {
        let z = -r + if ring == 0 { 0.0 } else { -r * 0.3 };
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([onr * a.cos(), onr * a.sin(), z]);
        }
    }
    let hr = res / 2;
    for i in 0..hr {
        let j = (i + 1) % hr;
        polys.push_cell(&[
            (onb + i) as i64,
            (onb + j) as i64,
            (onb + hr + j) as i64,
            (onb + hr + i) as i64,
        ]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let e = eye_model(1.2, 10);
        assert!(e.polys.num_cells() > 50);
    }
}
