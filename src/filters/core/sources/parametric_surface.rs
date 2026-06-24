//! Generic parametric surface from (u,v) -> (x,y,z) function.

use crate::data::{CellArray, Points, PolyData};

/// Create a parametric surface from a function f(u, v) -> [x, y, z].
pub fn parametric_surface(
    u_range: [f64; 2],
    v_range: [f64; 2],
    u_res: usize,
    v_res: usize,
    f: impl Fn(f64, f64) -> [f64; 3],
) -> PolyData {
    let ures = u_res.max(2);
    let vres = v_res.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for iv in 0..=vres {
        let v = v_range[0] + (v_range[1] - v_range[0]) * iv as f64 / vres as f64;
        for iu in 0..=ures {
            let u = u_range[0] + (u_range[1] - u_range[0]) * iu as f64 / ures as f64;
            pts.push(f(u, v));
        }
    }

    let w = ures + 1;
    for iv in 0..vres {
        for iu in 0..ures {
            let i00 = (iv * w + iu) as i64;
            let i10 = (iv * w + iu + 1) as i64;
            let i01 = ((iv + 1) * w + iu) as i64;
            let i11 = ((iv + 1) * w + iu + 1) as i64;
            polys.push_cell(&[i00, i10, i11, i01]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a closed parametric surface (wraps in both u and v).
pub fn parametric_surface_closed(
    u_res: usize,
    v_res: usize,
    f: impl Fn(f64, f64) -> [f64; 3],
) -> PolyData {
    let ures = u_res.max(3);
    let vres = v_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let pi2 = 2.0 * std::f64::consts::PI;

    for iv in 0..vres {
        let v = pi2 * iv as f64 / vres as f64;
        for iu in 0..ures {
            let u = pi2 * iu as f64 / ures as f64;
            pts.push(f(u, v));
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

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Example: Klein bottle.
pub fn klein_bottle(r: f64, res: usize) -> PolyData {
    parametric_surface_closed(res, res, |u, v| {
        let cu = u.cos();
        let su = u.sin();
        let cv = v.cos();
        let sv = v.sin();
        let x = (r + cu * sv.cos() - (u / 2.0).sin() * (2.0 * v).sin()) * cv;
        let y = (r + cu * sv.cos() - (u / 2.0).sin() * (2.0 * v).sin()) * sv;
        let z = su * sv.cos() + (u / 2.0).cos() * (2.0 * v).sin();
        [x, y, z]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_paraboloid() {
        let s = parametric_surface([-1.0, 1.0], [-1.0, 1.0], 10, 10, |u, v| {
            [u, v, u * u + v * v]
        });
        assert_eq!(s.points.len(), 121);
        assert_eq!(s.polys.num_cells(), 100);
    }
    #[test]
    fn test_closed() {
        let s = parametric_surface_closed(12, 24, |u, v| {
            let r = 2.0 + 0.5 * u.cos();
            [r * v.cos(), r * v.sin(), 0.5 * u.sin()]
        });
        assert_eq!(s.points.len(), 288);
        assert_eq!(s.polys.num_cells(), 288);
    }
    #[test]
    fn test_klein() {
        let k = klein_bottle(3.0, 16);
        assert_eq!(k.points.len(), 256);
    }
}
