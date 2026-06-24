//! Zigzag bridge (Japanese yatsuhashi style).
use crate::data::{CellArray, Points, PolyData};

pub fn zigzag_bridge(total_length: f64, width: f64, n_segments: usize) -> PolyData {
    let ns = n_segments.max(3);
    let seg_len = total_length / ns as f64;
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for s in 0..=ns {
        let x = seg_len * s as f64;
        let y_offset = if s % 2 == 0 { 0.0 } else { width * 0.8 };
        let z = 0.0;
        pts.push([x, y_offset - hw, z]);
        pts.push([x, y_offset + hw, z]);
    }
    for s in 0..ns {
        let b = s * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
    }
    // Railings
    let mut lines = CellArray::new();
    for s in 0..ns {
        let b = s * 2;
        lines.push_cell(&[b as i64, (b + 2) as i64]); // left rail
        lines.push_cell(&[(b + 1) as i64, (b + 3) as i64]); // right rail
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
    fn test_zigzag() {
        let m = zigzag_bridge(10.0, 1.5, 5);
        assert!(m.points.len() >= 12);
        assert!(m.polys.num_cells() == 5);
    }
}
