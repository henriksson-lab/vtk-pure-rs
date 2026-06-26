//! Chain link (torus-like interlocking ring) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn chain_link(major_radius: f64, minor_radius: f64, u_res: usize, v_res: usize) -> PolyData {
    let ures = u_res.max(3);
    let vres = v_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Elongated torus (stadium shape cross-section)
    let stretch = major_radius * 0.5;
    for iv in 0..vres {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / vres as f64;
        for iu in 0..ures {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ures as f64;
            let cx = (major_radius + stretch) * u.cos();
            let cy = major_radius * u.sin();
            let r = minor_radius;
            pts.push([
                cx + r * u.cos() * v.cos(),
                cy + r * u.sin() * v.cos(),
                r * v.sin(),
            ]);
        }
    }
    for iv in 0..vres {
        let iv1 = (iv + 1) % vres;
        for iu in 0..ures {
            let iu1 = (iu + 1) % ures;
            polys.push_cell(&[
                (iv * ures + iu) as i64,
                (iv * ures + iu1) as i64,
                (iv1 * ures + iu1) as i64,
                (iv1 * ures + iu) as i64,
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
    fn test_link() {
        let l = chain_link(1.0, 0.2, 16, 8);
        assert_eq!(l.points.len(), 128);
        assert_eq!(l.polys.num_cells(), 128);
    }
}
