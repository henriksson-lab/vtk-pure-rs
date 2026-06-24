//! Lunar base geometry (habitation domes + solar array + landing pad).
use crate::data::{CellArray, Points, PolyData};
pub fn lunar_base(
    dome_r: f64,
    num_domes: usize,
    _corridor_l: f64,
    pad_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nd = num_domes.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Habitation domes (hemispherical)
    for di in 0..nd {
        let dx = di as f64 * (dome_r * 2.5);
        let vres = res / 2;
        for iv in 0..=vres {
            let v = std::f64::consts::FRAC_PI_2 * iv as f64 / vres as f64;
            let sv = v.sin();
            let cv = v.cos();
            for iu in 0..=res {
                let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
                pts.push([
                    dx + dome_r * sv * u.cos(),
                    dome_r * sv * u.sin(),
                    dome_r * cv,
                ]);
            }
        }
        let w = res + 1;
        let base = pts.len() - (vres + 1) * w;
        for iv in 0..vres {
            for iu in 0..res {
                polys.push_cell(&[
                    (base + iv * w + iu) as i64,
                    (base + iv * w + iu + 1) as i64,
                    (base + (iv + 1) * w + iu + 1) as i64,
                    (base + (iv + 1) * w + iu) as i64,
                ]);
            }
        }
    }
    // Corridors between domes
    let ct = dome_r * 0.15;
    for di in 0..nd - 1 {
        let x0 = di as f64 * (dome_r * 2.5) + dome_r;
        let x1 = (di + 1) as f64 * (dome_r * 2.5) - dome_r;
        let cb = pts.len();
        pts.push([x0, -ct, 0.0]);
        pts.push([x1, -ct, 0.0]);
        pts.push([x1, ct, 0.0]);
        pts.push([x0, ct, 0.0]);
        pts.push([x0, -ct, ct * 2.0]);
        pts.push([x1, -ct, ct * 2.0]);
        pts.push([x1, ct, ct * 2.0]);
        pts.push([x0, ct, ct * 2.0]);
        let f = |i: usize| (cb + i) as i64;
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    }
    // Landing pad (flat circle)
    let pad_x = (nd as f64 - 1.0) * (dome_r * 2.5) / 2.0;
    let pad_y = -dome_r * 3.0;
    let pc = pts.len();
    pts.push([pad_x, pad_y, 0.0]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([pad_x + pad_r * a.cos(), pad_y + pad_r * a.sin(), 0.0]);
    }
    for i in 0..res {
        let j = if i + 1 < res { pc + 2 + i } else { pc + 1 };
        polys.push_cell(&[pc as i64, (pc + 1 + i) as i64, j as i64]);
    }
    // Solar array
    let sa_x = (nd as f64 - 1.0) * (dome_r * 2.5) / 2.0;
    let sa_y = dome_r * 2.5;
    let panel_w = dome_r * 3.0;
    let panel_h = dome_r * 1.5;
    let sb = pts.len();
    pts.push([sa_x - panel_w / 2.0, sa_y, dome_r * 0.3]);
    pts.push([sa_x + panel_w / 2.0, sa_y, dome_r * 0.3]);
    pts.push([sa_x + panel_w / 2.0, sa_y + panel_h, dome_r * 0.5]);
    pts.push([sa_x - panel_w / 2.0, sa_y + panel_h, dome_r * 0.5]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // Panel support
    let psb = pts.len();
    pts.push([sa_x, sa_y + panel_h / 2.0, 0.0]);
    pts.push([sa_x, sa_y + panel_h / 2.0, dome_r * 0.4]);
    lines.push_cell(&[psb as i64, (psb + 1) as i64]);
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
        let b = lunar_base(5.0, 3, 3.0, 8.0, 10);
        assert!(b.polys.num_cells() > 30);
    }
}
