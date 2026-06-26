//! Hexagonal segmented telescope mirror (like JWST).
use crate::data::{CellArray, Points, PolyData};

pub fn hex_telescope_array(segment_radius: f64, n_rings: usize) -> PolyData {
    let nr = n_rings.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let gap = segment_radius * 0.05;
    let spacing = segment_radius + gap;
    let axial_to_xy = |q: isize, r: isize| -> [f64; 2] {
        [
            spacing * 3.0f64.sqrt() * (q as f64 + r as f64 / 2.0),
            spacing * 1.5 * r as f64,
        ]
    };

    let mut centers = Vec::new();
    for q in -(nr as isize)..=(nr as isize) {
        for r in -(nr as isize)..=(nr as isize) {
            let s = -q - r;
            if q.abs().max(r.abs()).max(s.abs()) <= nr as isize {
                centers.push(axial_to_xy(q, r));
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
        assert_eq!(m.points.len(), 19 * 6);
        assert_eq!(m.polys.num_cells(), 19);
    }
}
