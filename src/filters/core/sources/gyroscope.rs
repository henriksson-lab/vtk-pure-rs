//! Gyroscope with spinning disk and gimbal rings.
use crate::data::{CellArray, Points, PolyData};

pub fn gyroscope(radius: f64, na: usize) -> PolyData {
    let na = na.max(16);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Spinning disk (filled circle in XZ plane)
    let dc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * 0.6 * a.cos(), 0.0, radius * 0.6 * a.sin()]);
    }
    for j in 0..na {
        polys.push_cell(&[
            dc as i64,
            (dc + 1 + j) as i64,
            (dc + 1 + (j + 1) % na) as i64,
        ]);
    }
    // Axle through disk
    let ax0 = pts.len();
    pts.push([0.0, -radius * 0.8, 0.0]);
    let ax1 = pts.len();
    pts.push([0.0, radius * 0.8, 0.0]);
    lines.push_cell(&[ax0 as i64, ax1 as i64]);
    // Inner gimbal ring (in XZ plane)
    let g1b = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * 0.8 * a.cos(), 0.0, radius * 0.8 * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(g1b + j) as i64, (g1b + (j + 1) % na) as i64]);
    }
    // Outer gimbal ring (in XY plane)
    let g2b = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        lines.push_cell(&[(g2b + j) as i64, (g2b + (j + 1) % na) as i64]);
    }
    // Stand
    let sb = pts.len();
    pts.push([0.0, 0.0, -radius * 1.3]);
    let bottom = g2b + 3 * na / 4;
    lines.push_cell(&[bottom as i64, sb as i64]); // bottom of outer ring to stand
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
    fn test_gyroscope() {
        let m = gyroscope(2.0, 16);
        assert!(m.points.len() > 40);
        assert!(m.lines.num_cells() > 20);
    }

    #[test]
    fn stand_attaches_to_bottom_of_outer_ring() {
        let radius = 2.0;
        let na = 16;
        let m = gyroscope(radius, na);
        let g2b = 1 + na + 2 + na;
        let bottom = g2b + 3 * na / 4;
        let stand = m.points.len() - 1;
        let line = m.lines.cell(m.lines.num_cells() - 1);
        let p = m.points.get(bottom);

        assert_eq!(line, &[bottom as i64, stand as i64]);
        assert!(p[1] < -radius + 1e-12);
    }
}
