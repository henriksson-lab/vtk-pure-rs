//! Maypole with ribbons.
use crate::data::{CellArray, Points, PolyData};

pub fn maypole(height: f64, n_ribbons: usize) -> PolyData {
    let nr = n_ribbons.max(4);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Pole
    let p0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let p1 = pts.len();
    pts.push([0.0, 0.0, height]);
    lines.push_cell(&[p0 as i64, p1 as i64]);
    // Crown at top (circle)
    let crown_r = height * 0.05;
    let na = 12;
    let cb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([crown_r * a.cos(), crown_r * a.sin(), height]);
    }
    for j in 0..na {
        lines.push_cell(&[(cb + j) as i64, (cb + (j + 1) % na) as i64]);
    }
    // Ribbons (helical curves from top to ground)
    let ribbon_r = height * 0.15;
    let n_pts_ribbon = 20;
    for r in 0..nr {
        let phase = 2.0 * std::f64::consts::PI * r as f64 / nr as f64;
        let rb = pts.len();
        for i in 0..=n_pts_ribbon {
            let t = i as f64 / n_pts_ribbon as f64;
            let z = height * (1.0 - t);
            let angle = phase + 3.0 * std::f64::consts::PI * t; // spiral down
            let r_expand = ribbon_r * (1.0 + 2.0 * t); // expand outward
            pts.push([r_expand * angle.cos(), r_expand * angle.sin(), z]);
        }
        for i in 0..n_pts_ribbon {
            lines.push_cell(&[(rb + i) as i64, (rb + i + 1) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_maypole() {
        let m = maypole(8.0, 6);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 100);
    }
}
