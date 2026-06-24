//! Torus knot and Lissajous knot tube geometry.

use crate::data::{CellArray, Points, PolyData};

/// Create a (p,q) torus knot tube.
pub fn torus_knot_tube(
    p: usize,
    q: usize,
    knot_radius: f64,
    tube_radius: f64,
    knot_res: usize,
    tube_res: usize,
) -> PolyData {
    let kres = knot_res.max(16);
    let tres = tube_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for ik in 0..kres {
        let t = 2.0 * std::f64::consts::PI * ik as f64 / kres as f64;
        let r = knot_radius * (1.0 + 0.5 * (q as f64 * t).cos());
        let cx = r * (p as f64 * t).cos();
        let cy = r * (p as f64 * t).sin();
        let cz = knot_radius * 0.5 * (q as f64 * t).sin();

        let t2 = 2.0 * std::f64::consts::PI * (ik as f64 + 0.01) / kres as f64;
        let r2 = knot_radius * (1.0 + 0.5 * (q as f64 * t2).cos());
        let tx = r2 * (p as f64 * t2).cos() - cx;
        let ty = r2 * (p as f64 * t2).sin() - cy;
        let tz = knot_radius * 0.5 * (q as f64 * t2).sin() - cz;
        let tlen = (tx * tx + ty * ty + tz * tz).sqrt().max(1e-15);
        let tang = [tx / tlen, ty / tlen, tz / tlen];

        let (n1, n2) = perp_frame(tang);
        for it in 0..tres {
            let a = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
            pts.push([
                cx + tube_radius * (a.cos() * n1[0] + a.sin() * n2[0]),
                cy + tube_radius * (a.cos() * n1[1] + a.sin() * n2[1]),
                cz + tube_radius * (a.cos() * n1[2] + a.sin() * n2[2]),
            ]);
        }
    }

    for ik in 0..kres {
        let r0 = ik * tres;
        let r1 = ((ik + 1) % kres) * tres;
        for it in 0..tres {
            let it1 = (it + 1) % tres;
            polys.push_cell(&[
                (r0 + it) as i64,
                (r0 + it1) as i64,
                (r1 + it1) as i64,
                (r1 + it) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a Lissajous knot tube.
pub fn lissajous_knot(
    a: f64,
    b: f64,
    c: f64,
    da: f64,
    db: f64,
    tube_radius: f64,
    res: usize,
    tube_res: usize,
) -> PolyData {
    let kres = res.max(16);
    let tres = tube_res.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for ik in 0..kres {
        let t = 2.0 * std::f64::consts::PI * ik as f64 / kres as f64;
        let cx = (a * t + da).cos();
        let cy = (b * t + db).cos();
        let cz = (c * t).cos();

        let t2 = 2.0 * std::f64::consts::PI * (ik as f64 + 0.01) / kres as f64;
        let tx = (a * t2 + da).cos() - cx;
        let ty = (b * t2 + db).cos() - cy;
        let tz = (c * t2).cos() - cz;
        let tlen = (tx * tx + ty * ty + tz * tz).sqrt().max(1e-15);
        let tang = [tx / tlen, ty / tlen, tz / tlen];
        let (n1, n2) = perp_frame(tang);

        for it in 0..tres {
            let ang = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
            pts.push([
                cx + tube_radius * (ang.cos() * n1[0] + ang.sin() * n2[0]),
                cy + tube_radius * (ang.cos() * n1[1] + ang.sin() * n2[1]),
                cz + tube_radius * (ang.cos() * n1[2] + ang.sin() * n2[2]),
            ]);
        }
    }

    for ik in 0..kres {
        let r0 = ik * tres;
        let r1 = ((ik + 1) % kres) * tres;
        for it in 0..tres {
            let it1 = (it + 1) % tres;
            polys.push_cell(&[
                (r0 + it) as i64,
                (r0 + it1) as i64,
                (r1 + it1) as i64,
                (r1 + it) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn perp_frame(t: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let up = if t[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let n1 = normalize(cross(t, up));
    let n2 = cross(t, n1);
    (n1, n2)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trefoil() {
        let k = torus_knot_tube(2, 3, 2.0, 0.2, 64, 8);
        assert_eq!(k.points.len(), 512);
        assert_eq!(k.polys.num_cells(), 512);
    }
    #[test]
    fn test_lissajous() {
        let k = lissajous_knot(2.0, 3.0, 5.0, 0.5, 0.7, 0.1, 64, 6);
        assert_eq!(k.points.len(), 384);
    }
}
