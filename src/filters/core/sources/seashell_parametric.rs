//! Parametric seashell (logarithmic spiral shell).
use crate::data::{CellArray, Points, PolyData};
pub fn parametric_seashell(
    turns: f64,
    opening_rate: f64,
    shell_radius: f64,
    tube_radius: f64,
    u_res: usize,
    v_res: usize,
) -> PolyData {
    let ur = u_res.max(16);
    let vr = v_res.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iu in 0..=ur {
        let u = turns * 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
        let r = shell_radius * (opening_rate * u).exp();
        let cx = r * u.cos();
        let cy = r * u.sin();
        let cz = u * 0.5;
        let tr = tube_radius * (opening_rate * u).exp();
        // Local frame along spiral
        let du = 0.001;
        let u2 = u + du;
        let r2 = shell_radius * (opening_rate * u2).exp();
        let tx = r2 * u2.cos() - cx;
        let ty = r2 * u2.sin() - cy;
        let tz = du * 0.5;
        let tl = (tx * tx + ty * ty + tz * tz).sqrt().max(1e-15);
        let tang = [tx / tl, ty / tl, tz / tl];
        let up = if tang[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let n1 = normalize(cross(tang, up));
        let n2 = cross(tang, n1);
        for iv in 0..vr {
            let v = 2.0 * std::f64::consts::PI * iv as f64 / vr as f64;
            pts.push([
                cx + tr * (v.cos() * n1[0] + v.sin() * n2[0]),
                cy + tr * (v.cos() * n1[1] + v.sin() * n2[1]),
                cz + tr * (v.cos() * n1[2] + v.sin() * n2[2]),
            ]);
        }
    }
    for iu in 0..ur {
        for iv in 0..vr {
            let iv1 = (iv + 1) % vr;
            polys.push_cell(&[
                (iu * vr + iv) as i64,
                (iu * vr + iv1) as i64,
                ((iu + 1) * vr + iv1) as i64,
                ((iu + 1) * vr + iv) as i64,
            ]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
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
    fn test() {
        let s = parametric_seashell(3.0, 0.1, 1.0, 0.3, 24, 8);
        assert!(s.polys.num_cells() > 100);
    }
}
