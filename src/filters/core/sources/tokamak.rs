//! Tokamak fusion reactor geometry (torus + magnets + ports).
use crate::data::{CellArray, Points, PolyData};
pub fn tokamak(
    major_r: f64,
    minor_r: f64,
    num_magnets: usize,
    port_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nm = num_magnets.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Plasma torus
    for iv in 0..res {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / res as f64;
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let r = major_r + minor_r * u.cos();
            pts.push([r * v.cos(), r * v.sin(), minor_r * u.sin()]);
        }
    }
    for iv in 0..res {
        let iv1 = (iv + 1) % res;
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iv * res + iu) as i64,
                (iv * res + iu1) as i64,
                (iv1 * res + iu1) as i64,
                (iv1 * res + iu) as i64,
            ]);
        }
    }
    // Toroidal field magnets (D-shaped coils)
    let coil_r = minor_r * 1.5;
    for mi in 0..nm {
        let a = 2.0 * std::f64::consts::PI * mi as f64 / nm as f64;
        let cx = major_r * a.cos();
        let cy = major_r * a.sin();
        let mut coil_ids = Vec::new();
        for ci in 0..res {
            let ca = 2.0 * std::f64::consts::PI * ci as f64 / res as f64;
            let x = cx + coil_r * ca.cos() * a.cos();
            let y = cy + coil_r * ca.cos() * a.sin();
            let z = coil_r * ca.sin();
            let idx = pts.len();
            pts.push([x, y, z]);
            coil_ids.push(idx as i64);
        }
        coil_ids.push(coil_ids[0]);
        lines.push_cell(&coil_ids);
    }
    // Diagnostic ports (cylinders extending outward)
    for pi in 0..4 {
        let a = std::f64::consts::FRAC_PI_2 * pi as f64;
        let px = (major_r + minor_r) * a.cos();
        let py = (major_r + minor_r) * a.sin();
        let dx = a.cos();
        let dy = a.sin();
        let pb = pts.len();
        for ring in 0..=1 {
            let ext = if ring == 0 { 0.0 } else { minor_r };
            for i in 0..res / 2 {
                let ca = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
                pts.push([
                    px + dx * ext + port_r * (-dy) * ca.cos(),
                    py + dy * ext + port_r * dx * ca.cos(),
                    port_r * ca.sin(),
                ]);
            }
        }
        let hr = res / 2;
        for i in 0..hr {
            let j = (i + 1) % hr;
            polys.push_cell(&[
                (pb + i) as i64,
                (pb + j) as i64,
                (pb + hr + j) as i64,
                (pb + hr + i) as i64,
            ]);
        }
    }
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
        let t = tokamak(10.0, 3.0, 8, 0.5, 8);
        assert!(t.polys.num_cells() > 50);
        assert!(t.lines.num_cells() >= 8);
    }
}
