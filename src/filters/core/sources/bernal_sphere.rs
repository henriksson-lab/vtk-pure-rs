//! Bernal sphere space habitat.
use crate::data::{CellArray, Points, PolyData};
pub fn bernal_sphere(radius: f64, window_strip_count: usize, resolution: usize) -> PolyData {
    let res = resolution.max(12);
    let nw = window_strip_count.max(0);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Main sphere
    let vres = res;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([radius * sv * u.cos(), radius * sv * u.sin(), radius * cv]);
        }
    }
    let w = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            // Skip window strips
            let is_window = nw > 0 && (iu % (res / nw.max(1))) == 0;
            if !is_window {
                polys.push_cell(&[
                    (iv * w + iu) as i64,
                    (iv * w + iu + 1) as i64,
                    ((iv + 1) * w + iu + 1) as i64,
                    ((iv + 1) * w + iu) as i64,
                ]);
            }
        }
    }
    // Agriculture ring (torus around equator)
    let ag_r = radius * 1.3;
    let ag_minor = radius * 0.15;
    let ag_base = pts.len();
    for iv in 0..res / 2 {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / (res / 2) as f64;
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let r = ag_r + ag_minor * u.cos();
            pts.push([r * v.cos(), r * v.sin(), ag_minor * u.sin()]);
        }
    }
    let hr = res / 2;
    for iv in 0..hr {
        let iv1 = (iv + 1) % hr;
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (ag_base + iv * res + iu) as i64,
                (ag_base + iv * res + iu1) as i64,
                (ag_base + iv1 * res + iu1) as i64,
                (ag_base + iv1 * res + iu) as i64,
            ]);
        }
    }
    // Sunlight mirrors (flat panels at poles)
    for &z_sign in &[1.0f64, -1.0] {
        let mz = z_sign * radius * 1.5;
        let mc = pts.len();
        pts.push([0.0, 0.0, mz]);
        let mr = radius * 0.8;
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([mr * a.cos(), mr * a.sin(), mz]);
        }
        for i in 0..res / 2 {
            let j = if i + 1 < res / 2 { mc + 2 + i } else { mc + 1 };
            polys.push_cell(&[mc as i64, (mc + 1 + i) as i64, j as i64]);
        }
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
        let b = bernal_sphere(100.0, 3, 12);
        assert!(b.polys.num_cells() > 100);
    }
}
