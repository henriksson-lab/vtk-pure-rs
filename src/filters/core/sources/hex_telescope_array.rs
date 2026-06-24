//! Hexagonal segmented telescope mirror (like JWST).
use crate::data::{CellArray, Points, PolyData};

pub fn hex_telescope_array(segment_radius: f64, n_rings: usize) -> PolyData {
    let nr = n_rings.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let gap = segment_radius * 0.05;
    let dx = (segment_radius + gap) * 3.0f64.sqrt();
    let dy = (segment_radius + gap) * 1.5;
    // Generate hex center positions
    let mut centers = vec![[0.0f64; 2]];
    for ring in 1..=nr {
        for side in 0..6 {
            for step in 0..ring {
                let _a0 = std::f64::consts::PI / 3.0 * side as f64;
                let _a1 = std::f64::consts::PI / 3.0 * ((side + 2) % 6) as f64;
                let cx = ring as f64
                    * dx
                    * (std::f64::consts::PI / 3.0 * side as f64 + std::f64::consts::PI / 6.0).cos()
                    + step as f64
                        * dx
                        * (std::f64::consts::PI / 3.0 * ((side + 2) % 6) as f64
                            + std::f64::consts::PI / 6.0)
                            .cos();
                let cy = ring as f64 * dy * 2.0 / 3.0
                    * (std::f64::consts::PI / 3.0 * side as f64 + std::f64::consts::PI / 6.0).sin()
                    + step as f64 * dy * 2.0 / 3.0
                        * (std::f64::consts::PI / 3.0 * ((side + 2) % 6) as f64
                            + std::f64::consts::PI / 6.0)
                            .sin();
                centers.push([cx, cy]);
            }
        }
    }
    // Place hexagonal segments at each center
    for &[cx, cy] in &centers {
        let base = pts.len();
        for j in 0..6 {
            let a = std::f64::consts::PI / 3.0 * j as f64 + std::f64::consts::PI / 6.0;
            pts.push([
                cx + segment_radius * a.cos(),
                cy + segment_radius * a.sin(),
                0.0,
            ]);
        }
        polys.push_cell(&[
            base as i64,
            (base + 1) as i64,
            (base + 2) as i64,
            (base + 3) as i64,
            (base + 4) as i64,
            (base + 5) as i64,
        ]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_array() {
        let m = hex_telescope_array(1.0, 2);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 5);
    }
}
