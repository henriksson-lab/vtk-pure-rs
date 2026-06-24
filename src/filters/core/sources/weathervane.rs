//! Weathervane (rooster/arrow on pivot with compass points).
use crate::data::{CellArray, Points, PolyData};

pub fn weathervane(height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Vertical pole
    let p0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let p1 = pts.len();
    pts.push([0.0, 0.0, height]);
    lines.push_cell(&[p0 as i64, p1 as i64]);
    // Compass cross (N/S/E/W arms)
    let arm_len = height * 0.15;
    let arm_z = height * 0.85;
    for &(dx, dy, _label) in &[
        (0.0, 1.0, "N"),
        (0.0, -1.0, "S"),
        (1.0, 0.0, "E"),
        (-1.0, 0.0, "W"),
    ] {
        let a0 = pts.len();
        pts.push([0.0, 0.0, arm_z]);
        let a1 = pts.len();
        pts.push([dx * arm_len, dy * arm_len, arm_z]);
        lines.push_cell(&[a0 as i64, a1 as i64]);
    }
    // Arrow pointer (triangle)
    let arrow_len = height * 0.3;
    let ab = pts.len();
    pts.push([0.0, -arrow_len * 0.3, height]);
    pts.push([0.0, arrow_len, height]);
    pts.push([0.0, arrow_len * 0.7, height + height * 0.05]);
    polys.push_cell(&[ab as i64, (ab + 1) as i64, (ab + 2) as i64]);
    // Tail (opposing triangle)
    let tb = pts.len();
    pts.push([0.0, -arrow_len * 0.3, height]);
    pts.push([0.0, -arrow_len, height]);
    pts.push([0.0, -arrow_len * 0.7, height + height * 0.08]);
    pts.push([0.0, -arrow_len * 0.7, height - height * 0.08]);
    polys.push_cell(&[tb as i64, (tb + 1) as i64, (tb + 2) as i64]);
    polys.push_cell(&[tb as i64, (tb + 1) as i64, (tb + 3) as i64]);
    // Decorative ball at top of pole (below arrow)
    let ball_r = height * 0.03;
    let na = 8;
    let bb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([ball_r * a.cos(), ball_r * a.sin(), height * 0.95]);
    }
    for j in 0..na {
        lines.push_cell(&[(bb + j) as i64, (bb + (j + 1) % na) as i64]);
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
    fn test_weathervane() {
        let m = weathervane(5.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() >= 3);
        assert!(m.lines.num_cells() > 5);
    }
}
