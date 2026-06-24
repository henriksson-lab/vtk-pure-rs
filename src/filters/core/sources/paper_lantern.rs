//! Paper lantern (ribbed sphere with top/bottom caps).
use crate::data::{CellArray, Points, PolyData};

pub fn paper_lantern(radius: f64, n_ribs: usize, n_parallels: usize) -> PolyData {
    let nr = n_ribs.max(6);
    let np = n_parallels.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Top and bottom attachment points
    let top = pts.len();
    pts.push([0.0, 0.0, radius]);
    let bot = pts.len();
    pts.push([0.0, 0.0, -radius]);
    // Ribs (meridian lines with slight bulge)
    for r in 0..nr {
        let theta = 2.0 * std::f64::consts::PI * r as f64 / nr as f64;
        let rib_base = pts.len();
        for p in 1..np {
            let phi = std::f64::consts::PI * p as f64 / np as f64;
            let bulge = 1.0 + 0.15 * (nr as f64 * theta + phi * 2.0).cos().abs();
            let rr = radius * phi.sin() * bulge;
            let z = radius * phi.cos();
            pts.push([rr * theta.cos(), rr * theta.sin(), z]);
        }
        // Lines along rib
        lines.push_cell(&[top as i64, rib_base as i64]);
        for p in 0..(np - 2) {
            lines.push_cell(&[(rib_base + p) as i64, (rib_base + p + 1) as i64]);
        }
        lines.push_cell(&[(rib_base + np - 2) as i64, bot as i64]);
    }
    // Surface panels between ribs
    for r in 0..nr {
        let r1 = (r + 1) % nr;
        let b0 = 2 + r * (np - 1);
        let b1 = 2 + r1 * (np - 1);
        // Top fan
        polys.push_cell(&[top as i64, b0 as i64, b1 as i64]);
        // Middle quads
        for p in 0..(np - 2) {
            polys.push_cell(&[
                (b0 + p) as i64,
                (b0 + p + 1) as i64,
                (b1 + p + 1) as i64,
                (b1 + p) as i64,
            ]);
        }
        // Bottom fan
        polys.push_cell(&[(b0 + np - 2) as i64, bot as i64, (b1 + np - 2) as i64]);
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
    fn test_lantern() {
        let m = paper_lantern(2.0, 8, 6);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 20);
        assert!(m.lines.num_cells() > 20);
    }
}
