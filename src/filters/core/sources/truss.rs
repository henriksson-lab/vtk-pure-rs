//! Simple truss/bridge structure geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn warren_truss(length: f64, height: f64, bays: usize) -> PolyData {
    let nb = bays.max(1);
    let bay_len = length / nb as f64;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Bottom chord
    for i in 0..=nb {
        pts.push([i as f64 * bay_len, 0.0, 0.0]);
    }
    // Top chord
    for i in 0..=nb {
        pts.push([i as f64 * bay_len, height, 0.0]);
    }
    let top_start = nb + 1;
    // Bottom chord edges
    for i in 0..nb {
        lines.push_cell(&[i as i64, (i + 1) as i64]);
    }
    // Top chord edges
    for i in 0..nb {
        lines.push_cell(&[(top_start + i) as i64, (top_start + i + 1) as i64]);
    }
    // Diagonals (Warren pattern)
    for i in 0..nb {
        lines.push_cell(&[i as i64, (top_start + i + 1) as i64]); // up-right
        lines.push_cell(&[(i + 1) as i64, (top_start + i) as i64]); // up-left
    }
    // Verticals at ends
    lines.push_cell(&[0, top_start as i64]);
    lines.push_cell(&[nb as i64, (top_start + nb) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn pratt_truss(length: f64, height: f64, bays: usize) -> PolyData {
    let nb = bays.max(1);
    let bay_len = length / nb as f64;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for i in 0..=nb {
        pts.push([i as f64 * bay_len, 0.0, 0.0]);
    }
    for i in 0..=nb {
        pts.push([i as f64 * bay_len, height, 0.0]);
    }
    let ts = nb + 1;
    for i in 0..nb {
        lines.push_cell(&[i as i64, (i + 1) as i64]);
    }
    for i in 0..nb {
        lines.push_cell(&[(ts + i) as i64, (ts + i + 1) as i64]);
    }
    // Verticals
    for i in 0..=nb {
        lines.push_cell(&[i as i64, (ts + i) as i64]);
    }
    // Diagonals (Pratt: from bottom outer to top inner)
    let mid = nb / 2;
    for i in 0..mid {
        lines.push_cell(&[i as i64, (ts + i + 1) as i64]);
    }
    for i in mid..nb {
        lines.push_cell(&[(i + 1) as i64, (ts + i) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_warren() {
        let t = warren_truss(10.0, 2.0, 5);
        assert!(t.lines.num_cells() > 10);
    }
    #[test]
    fn test_pratt() {
        let t = pratt_truss(10.0, 2.0, 6);
        assert!(t.lines.num_cells() > 10);
    }
}
