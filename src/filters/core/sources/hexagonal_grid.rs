//! Hexagonal grid surface.
use crate::data::{CellArray, Points, PolyData};
pub fn hex_grid(cols: usize, rows: usize, size: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let size = size.abs();
    let dx = size * 1.5;
    let dy = size * 3.0f64.sqrt();
    for r in 0..rows {
        for c in 0..cols {
            let cx = c as f64 * dx;
            let cy = r as f64 * dy + (if c % 2 == 1 { dy / 2.0 } else { 0.0 });
            let b = pts.len();
            for k in 0..6 {
                let a = std::f64::consts::PI / 3.0 * k as f64;
                pts.push([cx + size * a.cos(), cy + size * a.sin(), 0.0]);
            }
            polys.push_cell(&(0..6).map(|k| (b + k) as i64).collect::<Vec<_>>());
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let h = hex_grid(5, 4, 1.0);
        assert_eq!(h.polys.num_cells(), 20);
        assert_eq!(h.points.len(), 120);
    }
}
