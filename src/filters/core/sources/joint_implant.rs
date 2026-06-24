//! Hip joint implant (femoral stem + ball + acetabular cup).
use crate::data::{CellArray, Points, PolyData};
pub fn hip_implant(
    ball_r: f64,
    stem_l: f64,
    stem_r: f64,
    cup_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Femoral ball (hemisphere)
    let vres = res / 2;
    for iv in 0..=vres {
        let v = std::f64::consts::FRAC_PI_2 * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([ball_r * sv * u.cos(), ball_r * sv * u.sin(), ball_r * cv]);
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
    // Neck (tapered cylinder connecting ball to stem)
    let neck_l = ball_r * 1.5;
    let nb = pts.len();
    for ring in 0..=1 {
        let t = ring as f64;
        let r = ball_r * (1.0 - t * 0.3);
        let z = -neck_l * t;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (nb + i) as i64,
            (nb + j) as i64,
            (nb + res + j) as i64,
            (nb + res + i) as i64,
        ]);
    }
    // Stem (tapered rod)
    let sb = pts.len();
    let nseg = 4;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let r = stem_r * (1.0 - t * 0.5);
        let z = -neck_l - stem_l * t;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for is in 0..nseg {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (sb + is * res + i) as i64,
                (sb + is * res + j) as i64,
                (sb + (is + 1) * res + j) as i64,
                (sb + (is + 1) * res + i) as i64,
            ]);
        }
    }
    // Acetabular cup (inverted hemisphere, positioned above ball)
    let cup_z = ball_r * 1.1;
    let cb = pts.len();
    for iv in 0..=vres {
        let v = std::f64::consts::FRAC_PI_2 * iv as f64 / vres as f64;
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([
                cup_r * v.sin() * u.cos(),
                cup_r * v.sin() * u.sin(),
                cup_z + cup_r * v.cos(),
            ]);
        }
    }
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (cb + (iv + 1) * w + iu) as i64,
                (cb + (iv + 1) * w + iu + 1) as i64,
                (cb + iv * w + iu + 1) as i64,
                (cb + iv * w + iu) as i64,
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
        let h = hip_implant(0.028, 0.15, 0.012, 0.03, 8);
        assert!(h.polys.num_cells() > 40);
    }
}
