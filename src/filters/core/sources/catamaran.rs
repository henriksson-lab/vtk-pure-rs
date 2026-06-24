//! Catamaran (twin-hull sailboat).
use crate::data::{CellArray, Points, PolyData};

pub fn catamaran(length: f64, beam: f64, hull_width: f64, ns: usize) -> PolyData {
    let ns = ns.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hull_separation = beam / 2.0;
    // Two hulls
    for &side in &[-1.0f64, 1.0] {
        let cy = side * hull_separation;
        let nw = 3;
        for s in 0..=ns {
            let t = s as f64 / ns as f64;
            let x = length * (t - 0.5);
            let taper = (std::f64::consts::PI * t).sin().max(0.05);
            let hw = hull_width / 2.0 * taper;
            for w in 0..=nw {
                let u = w as f64 / nw as f64;
                let angle = std::f64::consts::PI * u;
                let y = cy + hw * angle.cos();
                let z = -hull_width * 0.3 * angle.sin() * taper;
                pts.push([x, y, z]);
            }
        }
    }
    let stride = 4; // nw+1
    for hull in 0..2 {
        let base = hull * (ns + 1) * stride;
        for s in 0..ns {
            for w in 0..3 {
                let i0 = base + s * stride + w;
                let i1 = base + (s + 1) * stride + w;
                polys.push_cell(&[i0 as i64, i1 as i64, (i1 + 1) as i64, (i0 + 1) as i64]);
            }
        }
    }
    // Cross beams connecting hulls
    let n_beams = 3;
    for b in 0..n_beams {
        let t = (b + 1) as f64 / (n_beams + 1) as f64;
        let x = length * (t - 0.5);
        let l = pts.len();
        pts.push([x, -hull_separation, 0.02]);
        let r = pts.len();
        pts.push([x, hull_separation, 0.02]);
        lines.push_cell(&[l as i64, r as i64]);
    }
    // Mast
    let mast_b = pts.len();
    pts.push([0.0, 0.0, 0.02]);
    let mast_t = pts.len();
    pts.push([0.0, 0.0, length * 0.6]);
    lines.push_cell(&[mast_b as i64, mast_t as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_catamaran() {
        let m = catamaran(10.0, 4.0, 0.8, 10);
        assert!(m.points.len() > 60);
        assert!(m.polys.num_cells() > 30);
    }
}
