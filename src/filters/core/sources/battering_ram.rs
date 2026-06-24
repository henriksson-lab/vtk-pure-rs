//! Medieval battering ram (log with head, suspended in frame).
use crate::data::{CellArray, Points, PolyData};

pub fn battering_ram(length: f64, ram_radius: f64, frame_height: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Ram log (cylinder along X)
    let segs = 6;
    for s in 0..=segs {
        let x = -length / 2.0 + length * s as f64 / segs as f64;
        let r = if s == 0 { ram_radius * 1.3 } else { ram_radius }; // head is thicker
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, r * a.cos(), frame_height + r * a.sin()]);
        }
    }
    for s in 0..segs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // A-frame supports (two pairs)
    for &x in &[-length * 0.3, length * 0.3] {
        let spread = frame_height * 0.4;
        let t = pts.len();
        pts.push([x, 0.0, frame_height + ram_radius * 1.5]);
        let bl = pts.len();
        pts.push([x, -spread, 0.0]);
        let br = pts.len();
        pts.push([x, spread, 0.0]);
        lines.push_cell(&[t as i64, bl as i64]);
        lines.push_cell(&[t as i64, br as i64]);
        // Suspension rope
        let ram_pt = (((x + length / 2.0) / length * segs as f64).round() as usize).min(segs) * na;
        lines.push_cell(&[t as i64, ram_pt as i64]);
    }
    // Cross beam at top
    let cl = pts.len();
    pts.push([-length * 0.3, 0.0, frame_height + ram_radius * 1.5]);
    let cr = pts.len();
    pts.push([length * 0.3, 0.0, frame_height + ram_radius * 1.5]);
    lines.push_cell(&[cl as i64, cr as i64]);
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
    fn test_ram() {
        let m = battering_ram(6.0, 0.3, 2.0, 10);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 40);
        assert!(m.lines.num_cells() > 5);
    }
}
