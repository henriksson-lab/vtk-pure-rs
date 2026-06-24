//! Portcullis (castle gate) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn portcullis(width: f64, height: f64, bar_spacing: f64, _bar_radius: f64) -> PolyData {
    let hw = width / 2.0;
    let nb = ((width / bar_spacing).ceil() as usize).max(2);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Vertical bars
    for i in 0..nb {
        let x = -hw + i as f64 * width / (nb - 1) as f64;
        let b = pts.len();
        pts.push([x, 0.0, 0.0]);
        pts.push([x, 0.0, height]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Horizontal bars
    let nh = ((height / bar_spacing).ceil() as usize).max(2);
    for i in 0..nh {
        let z = i as f64 * height / (nh - 1) as f64;
        let b = pts.len();
        pts.push([-hw, 0.0, z]);
        pts.push([hw, 0.0, z]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Spikes at bottom
    for i in 0..nb {
        let x = -hw + i as f64 * width / (nb - 1) as f64;
        let b = pts.len();
        pts.push([x, 0.0, 0.0]);
        pts.push([x, 0.0, -bar_spacing * 0.5]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
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
    fn test() {
        let p = portcullis(3.0, 4.0, 0.3, 0.03);
        assert!(p.lines.num_cells() > 15);
    }
}
