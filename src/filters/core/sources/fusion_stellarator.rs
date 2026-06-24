//! Stellarator fusion reactor (twisted torus).
use crate::data::{CellArray, Points, PolyData};
pub fn stellarator(
    major_r: f64,
    minor_r: f64,
    twist_periods: usize,
    twist_amplitude: f64,
    u_res: usize,
    v_res: usize,
) -> PolyData {
    let ur = u_res.max(12);
    let vr = v_res.max(8);
    let np = twist_periods.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..vr {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / vr as f64;
        for iu in 0..ur {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            // Twisted torus: cross-section rotates and deforms
            let twist_angle = np as f64 * v;
            let deform = 1.0 + twist_amplitude * (twist_angle).cos();
            let r = major_r + minor_r * deform * (u + twist_angle * 0.3).cos();
            let x = r * v.cos();
            let y = r * v.sin();
            let z = minor_r * deform * (u + twist_angle * 0.3).sin();
            pts.push([x, y, z]);
        }
    }
    for iv in 0..vr {
        let iv1 = (iv + 1) % vr;
        for iu in 0..ur {
            let iu1 = (iu + 1) % ur;
            polys.push_cell(&[
                (iv * ur + iu) as i64,
                (iv * ur + iu1) as i64,
                (iv1 * ur + iu1) as i64,
                (iv1 * ur + iu) as i64,
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
        let s = stellarator(10.0, 2.0, 5, 0.3, 24, 12);
        assert_eq!(s.polys.num_cells(), 288);
    }
}
