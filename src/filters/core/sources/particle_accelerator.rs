//! Particle accelerator ring geometry (synchrotron).
use crate::data::{CellArray, Points, PolyData};
pub fn synchrotron_ring(
    ring_r: f64,
    tunnel_r: f64,
    num_magnets: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(num_magnets * 4);
    let tres = resolution.max(6);
    let nm = num_magnets.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Beam pipe (torus)
    for iv in 0..res {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / res as f64;
        let cx = (ring_r) * v.cos();
        let cy = (ring_r) * v.sin();
        for iu in 0..tres {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / tres as f64;
            let r = tunnel_r * 0.3; // beam pipe is smaller than tunnel
            pts.push([
                cx + r * u.cos() * v.cos(),
                cy + r * u.cos() * v.sin(),
                r * u.sin(),
            ]);
        }
    }
    for iv in 0..res {
        let iv1 = (iv + 1) % res;
        for iu in 0..tres {
            let iu1 = (iu + 1) % tres;
            polys.push_cell(&[
                (iv * tres + iu) as i64,
                (iv * tres + iu1) as i64,
                (iv1 * tres + iu1) as i64,
                (iv1 * tres + iu) as i64,
            ]);
        }
    }
    // Magnets (boxes around the ring at intervals)
    let mag_size = tunnel_r * 0.8;
    for mi in 0..nm {
        let a = 2.0 * std::f64::consts::PI * mi as f64 / nm as f64;
        let cx = ring_r * a.cos();
        let cy = ring_r * a.sin();
        // Tangent direction
        let tx = -a.sin();
        let ty = a.cos();
        let nx = a.cos();
        let ny = a.sin();
        let ms = mag_size;
        let ml = ring_r * 2.0 * std::f64::consts::PI / (nm * 3) as f64;
        let mb = pts.len();
        // 8 corners of magnet box
        for &dz in &[-ms, ms] {
            for &dt in &[-ml / 2.0, ml / 2.0] {
                for &dn in &[-ms, ms] {
                    pts.push([cx + tx * dt + nx * dn, cy + ty * dt + ny * dn, dz]);
                }
            }
        }
        let f = |i: usize| (mb + i) as i64;
        polys.push_cell(&[f(0), f(2), f(6), f(4)]);
        polys.push_cell(&[f(1), f(5), f(7), f(3)]);
        polys.push_cell(&[f(0), f(1), f(3), f(2)]);
        polys.push_cell(&[f(4), f(6), f(7), f(5)]);
        polys.push_cell(&[f(0), f(4), f(5), f(1)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    }
    // Detector halls (larger boxes at 4 positions)
    for di in 0..4 {
        let a = std::f64::consts::FRAC_PI_2 * di as f64;
        let cx = ring_r * a.cos();
        let cy = ring_r * a.sin();
        let ds = tunnel_r * 2.0;
        let db = pts.len();
        pts.push([cx - ds, cy - ds, -ds]);
        pts.push([cx + ds, cy - ds, -ds]);
        pts.push([cx + ds, cy + ds, -ds]);
        pts.push([cx - ds, cy + ds, -ds]);
        pts.push([cx - ds, cy - ds, ds]);
        pts.push([cx + ds, cy - ds, ds]);
        pts.push([cx + ds, cy + ds, ds]);
        pts.push([cx - ds, cy + ds, ds]);
        let g = |i: usize| (db + i) as i64;
        polys.push_cell(&[g(0), g(3), g(2), g(1)]);
        polys.push_cell(&[g(4), g(5), g(6), g(7)]);
        polys.push_cell(&[g(0), g(1), g(5), g(4)]);
        polys.push_cell(&[g(2), g(3), g(7), g(6)]);
        polys.push_cell(&[g(0), g(4), g(7), g(3)]);
        polys.push_cell(&[g(1), g(2), g(6), g(5)]);
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
        let s = synchrotron_ring(100.0, 3.0, 8, 16);
        assert!(s.polys.num_cells() > 100);
    }
}
