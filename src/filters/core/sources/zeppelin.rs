//! Zeppelin airship with gondola and fins.
use crate::data::{CellArray, Points, PolyData};

pub fn zeppelin(length: f64, diameter: f64, na: usize, ns: usize) -> PolyData {
    let na = na.max(12);
    let ns = ns.max(8);
    let r = diameter / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Envelope (ellipsoid-like)
    let top = pts.len();
    pts.push([length / 2.0, 0.0, 0.0]); // nose
    for s in 1..ns {
        let t = s as f64 / ns as f64;
        let x = length * (0.5 - t);
        let profile_r = r * (4.0 * t * (1.0 - t)).sqrt(); // elliptical
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, profile_r * a.cos(), profile_r * a.sin()]);
        }
    }
    let tail = pts.len();
    pts.push([-length / 2.0, 0.0, 0.0]); // tail
                                         // Nose cap
    for j in 0..na {
        polys.push_cell(&[
            top as i64,
            (top + 1 + j) as i64,
            (top + 1 + (j + 1) % na) as i64,
        ]);
    }
    // Body bands
    for s in 0..(ns - 2) {
        let b0 = top + 1 + s * na;
        let b1 = top + 1 + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Tail cap
    let last_ring = top + 1 + (ns - 2) * na;
    for j in 0..na {
        polys.push_cell(&[
            (last_ring + j) as i64,
            tail as i64,
            (last_ring + (j + 1) % na) as i64,
        ]);
    }
    // Gondola (small box underneath)
    let gw = length * 0.08;
    let gh = diameter * 0.15;
    let gl = length * 0.2;
    let gb = pts.len();
    pts.push([-gl / 2.0, -r - gh, -gw]);
    pts.push([gl / 2.0, -r - gh, -gw]);
    pts.push([gl / 2.0, -r - gh, gw]);
    pts.push([-gl / 2.0, -r - gh, gw]);
    pts.push([-gl / 2.0, -r, -gw]);
    pts.push([gl / 2.0, -r, -gw]);
    pts.push([gl / 2.0, -r, gw]);
    pts.push([-gl / 2.0, -r, gw]);
    polys.push_cell(&[gb as i64, (gb + 1) as i64, (gb + 5) as i64, (gb + 4) as i64]);
    polys.push_cell(&[
        (gb + 1) as i64,
        (gb + 2) as i64,
        (gb + 6) as i64,
        (gb + 5) as i64,
    ]);
    polys.push_cell(&[
        (gb + 2) as i64,
        (gb + 3) as i64,
        (gb + 7) as i64,
        (gb + 6) as i64,
    ]);
    polys.push_cell(&[(gb + 3) as i64, gb as i64, (gb + 4) as i64, (gb + 7) as i64]);
    polys.push_cell(&[gb as i64, (gb + 3) as i64, (gb + 2) as i64, (gb + 1) as i64]); // bottom
                                                                                      // Tail fins (4 triangles)
    for &(dy, dz) in &[(0.0, 1.0), (0.0, -1.0), (1.0, 0.0), (-1.0, 0.0)] {
        let fb = pts.len();
        pts.push([-length * 0.35, dy * r * 0.1, dz * r * 0.1]);
        pts.push([-length * 0.5, dy * r * 0.1, dz * r * 0.1]);
        pts.push([-length * 0.5, dy * r * 0.5, dz * r * 0.5]);
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64]);
    }
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
    fn test_zeppelin() {
        let m = zeppelin(20.0, 5.0, 16, 10);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 80);
    }
}
