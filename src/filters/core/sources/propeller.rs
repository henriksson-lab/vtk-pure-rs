//! Propeller/fan blade geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn propeller(
    num_blades: usize,
    blade_length: f64,
    blade_width: f64,
    hub_radius: f64,
    pitch_angle: f64,
    resolution: usize,
) -> PolyData {
    let nb = num_blades.max(2);
    let res = resolution.max(4);
    let bw = blade_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let pa = pitch_angle.to_radians();
    // Hub disk
    let hc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..nb * 4 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / (nb * 4) as f64;
        pts.push([hub_radius * a.cos(), hub_radius * a.sin(), 0.0]);
    }
    for i in 0..nb * 4 {
        let j = if i + 1 < nb * 4 { hc + 2 + i } else { hc + 1 };
        polys.push_cell(&[hc as i64, (hc + 1 + i) as i64, j as i64]);
    }
    // Blades
    for bi in 0..nb {
        let base_angle = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64;
        for ri in 0..res {
            let t0 = ri as f64 / res as f64;
            let t1 = (ri + 1) as f64 / res as f64;
            let r0 = hub_radius + blade_length * t0;
            let r1 = hub_radius + blade_length * t1;
            let twist0 = pa * t0;
            let twist1 = pa * t1;
            let ca = base_angle.cos();
            let sa = base_angle.sin();
            let b = pts.len();
            pts.push([
                r0 * ca - bw * twist0.cos() * sa,
                r0 * sa + bw * twist0.cos() * ca,
                bw * twist0.sin(),
            ]);
            pts.push([
                r0 * ca + bw * twist0.cos() * sa,
                r0 * sa - bw * twist0.cos() * ca,
                -bw * twist0.sin(),
            ]);
            pts.push([
                r1 * ca + bw * twist1.cos() * sa,
                r1 * sa - bw * twist1.cos() * ca,
                -bw * twist1.sin(),
            ]);
            pts.push([
                r1 * ca - bw * twist1.cos() * sa,
                r1 * sa + bw * twist1.cos() * ca,
                bw * twist1.sin(),
            ]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
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
        let p = propeller(3, 2.0, 0.3, 0.3, 15.0, 6);
        assert!(p.polys.num_cells() > 15);
    }
}
