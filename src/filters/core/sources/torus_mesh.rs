//! Torus mesh with configurable resolution.
use crate::data::{CellArray, Points, PolyData};
pub fn torus_mesh(major_radius: f64, minor_radius: f64, u_res: usize, v_res: usize) -> PolyData {
    let ur = u_res.max(3);
    let vr = v_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..vr {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / vr as f64;
        for iu in 0..ur {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            let r = major_radius + minor_radius * u.cos();
            pts.push([r * v.cos(), r * v.sin(), minor_radius * u.sin()]);
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
        let t = torus_mesh(2.0, 0.5, 24, 12);
        assert_eq!(t.points.len(), 288);
        assert_eq!(t.polys.num_cells(), 288);
    }
}
