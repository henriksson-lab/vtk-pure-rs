//! Spinning top (conical body with handle).
use crate::data::{CellArray, Points, PolyData};

pub fn spinning_top(radius: f64, height: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Tip
    let tip = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    // Body (expanding then contracting)
    let profile = [
        (0.0, 0.0),
        (0.3, 0.5),
        (0.8, 1.0),
        (1.0, 0.9),
        (0.7, 0.7),
        (0.3, 0.6),
        (0.15, 0.55),
    ];
    for &(h_frac, r_frac) in &profile {
        let z = height * h_frac;
        let r = radius * r_frac;
        if r < 1e-10 {
            continue;
        } // skip zero-radius (use tip instead)
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    // Tip to first ring
    let first_ring = tip + 1;
    for j in 0..na {
        polys.push_cell(&[
            tip as i64,
            (first_ring + j) as i64,
            (first_ring + (j + 1) % na) as i64,
        ]);
    }
    // Connect rings
    let n_rings = profile.len() - 1; // minus the tip
    for ri in 0..(n_rings - 1) {
        let b0 = first_ring + ri * na;
        let b1 = first_ring + (ri + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Handle (line extending up)
    let mut lines = CellArray::new();
    let hb = pts.len();
    pts.push([0.0, 0.0, height * 0.55]);
    let ht = pts.len();
    pts.push([0.0, 0.0, height * 0.8]);
    lines.push_cell(&[hb as i64, ht as i64]);
    // Handle knob
    let knob = pts.len();
    pts.push([0.0, 0.0, height * 0.85]);
    let knob_r = radius * 0.2;
    let kb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([knob_r * a.cos(), knob_r * a.sin(), height * 0.8]);
    }
    for j in 0..na {
        polys.push_cell(&[(kb + j) as i64, knob as i64, (kb + (j + 1) % na) as i64]);
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
    fn test_top() {
        let m = spinning_top(2.0, 4.0, 16);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 40);
    }
}
