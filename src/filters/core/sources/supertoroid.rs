//! Supertoroid geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a supertoroid with exponents n1 and n2.
/// n1 controls the ring shape in the x-y plane, n2 controls the cross-section shape.
/// n1=n2=1.0 gives a regular torus.
pub fn supertoroid(
    major_radius: f64,
    minor_radius: f64,
    n1: f64,
    n2: f64,
    u_res: usize,
    v_res: usize,
) -> PolyData {
    let ures = u_res.max(3);
    let vres = v_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for iv in 0..vres {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / vres as f64;
        for iu in 0..ures {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ures as f64;
            let cu = signed_pow(u.cos(), n1);
            let su = signed_pow(u.sin(), n1);
            let cv = signed_pow(v.cos(), n2);
            let sv = signed_pow(v.sin(), n2);
            let tmp = major_radius + minor_radius * cv;
            let x = tmp * su;
            let y = tmp * cu;
            let z = minor_radius * sv;
            pts.push([x, y, z]);
        }
    }

    for iv in 0..vres {
        let iv1 = (iv + 1) % vres;
        for iu in 0..ures {
            let iu1 = (iu + 1) % ures;
            let i00 = (iv * ures + iu) as i64;
            let i10 = (iv * ures + iu1) as i64;
            let i01 = (iv1 * ures + iu) as i64;
            let i11 = (iv1 * ures + iu1) as i64;
            polys.push_cell(&[i00, i10, i11, i01]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn signed_pow(val: f64, exp: f64) -> f64 {
    if val == 0.0 {
        return 0.0;
    }
    if exp == 0.0 {
        return 1.0;
    }
    val.signum() * val.abs().powf(exp)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_regular_torus() {
        let t = supertoroid(2.0, 0.5, 1.0, 1.0, 16, 32);
        assert_eq!(t.points.len(), 512);
        assert_eq!(t.polys.num_cells(), 512);
    }
    #[test]
    fn test_square_cross() {
        let t = supertoroid(2.0, 0.5, 0.2, 1.0, 16, 32);
        assert_eq!(t.points.len(), 512);
    }
    #[test]
    fn test_square_ring() {
        let t = supertoroid(2.0, 0.5, 1.0, 0.2, 8, 8);
        assert_eq!(t.points.len(), 64);
    }
}
