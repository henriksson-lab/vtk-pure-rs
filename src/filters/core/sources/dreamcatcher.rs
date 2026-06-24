//! Dreamcatcher (hoop with web and feathers).
use crate::data::{CellArray, Points, PolyData};

pub fn dreamcatcher(radius: f64, n_web_rings: usize, na: usize) -> PolyData {
    let nw = n_web_rings.max(3);
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Outer hoop
    let hb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % na) as i64]);
    }
    // Web (concentric polygons getting smaller and rotated)
    for w in 1..=nw {
        let r = radius * (1.0 - w as f64 / (nw + 1) as f64);
        let rot = 0.1 * w as f64;
        let wb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64 + rot;
            pts.push([r * a.cos(), r * a.sin(), 0.0]);
        }
        for j in 0..na {
            lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % na) as i64]);
        }
        // Connect to previous ring
        let prev = if w == 1 { hb } else { hb + (w - 1) * na };
        for j in 0..na {
            lines.push_cell(&[(prev + j) as i64, (wb + j) as i64]);
        }
    }
    // Feathers (hanging lines from bottom)
    for f in 0..3 {
        let angle = std::f64::consts::PI + std::f64::consts::PI * 0.3 * (f as f64 - 1.0);
        let attach_j = (angle / (2.0 * std::f64::consts::PI) * na as f64).round() as usize % na;
        let fb = pts.len();
        pts.push([
            radius * 1.1 * (angle).cos(),
            radius * 1.1 * (angle).sin(),
            -radius * 0.3 * (f as f64 + 1.0) / 3.0,
        ]);
        lines.push_cell(&[(hb + attach_j) as i64, fb as i64]);
        // Feather barbs
        let fe = pts.len();
        pts.push([
            radius * 1.15 * (angle).cos(),
            radius * 1.15 * (angle).sin(),
            -radius * 0.4 * (f as f64 + 1.0) / 3.0,
        ]);
        lines.push_cell(&[fb as i64, fe as i64]);
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
    fn test_dreamcatcher() {
        let m = dreamcatcher(3.0, 4, 16);
        assert!(m.points.len() > 60);
        assert!(m.lines.num_cells() > 60);
    }
}
