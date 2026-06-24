//! Klein bottle immersed in 3D (figure-8 immersion).
use crate::data::{CellArray, Points, PolyData};
pub fn klein_bottle_figure8(scale: f64, u_res: usize, v_res: usize) -> PolyData {
    let ur = u_res.max(8);
    let vr = v_res.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..vr {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / vr as f64;
        for iu in 0..ur {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            let cu = u.cos();
            let su = u.sin();
            let cv = v.cos();
            let sv = v.sin();
            let r = scale * (4.0 + 2.0 * cu * (u / 2.0).cos() - su * (u / 2.0).sin());
            let x = r * cv;
            let y = r * sv;
            let z = scale * (2.0 * su * (u / 2.0).cos() + cu * (u / 2.0).sin());
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
        let k = klein_bottle_figure8(1.0, 16, 16);
        assert_eq!(k.polys.num_cells(), 256);
    }
}
