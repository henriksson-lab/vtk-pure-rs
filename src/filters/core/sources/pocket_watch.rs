//! Pocket watch with case, face, and chain.
use crate::data::{CellArray, Points, PolyData};

pub fn pocket_watch(radius: f64, na: usize) -> PolyData {
    let na = na.max(16);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Case (ring)
    let case_outer = radius * 1.1;
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([case_outer * a.cos(), case_outer * a.sin(), -0.02]);
        pts.push([case_outer * a.cos(), case_outer * a.sin(), 0.02]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (j * 2) as i64,
            (j * 2 + 1) as i64,
            (j1 * 2 + 1) as i64,
            (j1 * 2) as i64,
        ]);
    }
    // Face (filled circle)
    let fc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let fb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[fc as i64, (fb + j) as i64, (fb + (j + 1) % na) as i64]);
    }
    // Hour markers
    for h in 0..12 {
        let a = 2.0 * std::f64::consts::PI * h as f64 / 12.0;
        let m0 = pts.len();
        pts.push([radius * 0.85 * a.cos(), radius * 0.85 * a.sin(), 0.01]);
        let m1 = pts.len();
        pts.push([radius * 0.95 * a.cos(), radius * 0.95 * a.sin(), 0.01]);
        lines.push_cell(&[m0 as i64, m1 as i64]);
    }
    // Hands
    let hc = pts.len();
    pts.push([0.0, 0.0, 0.01]);
    let hh = pts.len();
    pts.push([0.0, radius * 0.5, 0.01]); // hour
    let mh = pts.len();
    pts.push([radius * 0.6, radius * 0.2, 0.01]); // minute
    lines.push_cell(&[hc as i64, hh as i64]);
    lines.push_cell(&[hc as i64, mh as i64]);
    // Crown (small bump at top)
    let crown = pts.len();
    pts.push([0.0, case_outer + 0.1, 0.0]);
    lines.push_cell(&[(fb + na / 4) as i64, crown as i64]); // connect to top of case
                                                            // Chain
    let chain_end = pts.len();
    pts.push([0.0, case_outer + radius, 0.0]);
    lines.push_cell(&[crown as i64, chain_end as i64]);
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
    fn test_watch() {
        let m = pocket_watch(2.0, 24);
        assert!(m.points.len() > 60);
        assert!(m.polys.num_cells() > 20);
        assert!(m.lines.num_cells() > 12);
    }
}
