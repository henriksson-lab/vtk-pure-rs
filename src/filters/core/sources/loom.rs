//! Weaving loom (frame with warp threads and heddles).
use crate::data::{CellArray, Points, PolyData};

pub fn loom(width: f64, height: f64, depth: f64, n_warp: usize) -> PolyData {
    let nw = n_warp.max(5);
    let hw = width / 2.0;
    let _hh = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Frame uprights (4 corners)
    for &x in &[-hw, hw] {
        for &y in &[0.0f64, depth] {
            let b = pts.len();
            pts.push([x, y, 0.0]);
            let t = pts.len();
            pts.push([x, y, height]);
            lines.push_cell(&[b as i64, t as i64]);
        }
    }
    // Top beams
    for &z in &[0.0, height] {
        let b0 = pts.len();
        pts.push([-hw, 0.0, z]);
        let b1 = pts.len();
        pts.push([hw, 0.0, z]);
        lines.push_cell(&[b0 as i64, b1 as i64]);
        let b2 = pts.len();
        pts.push([-hw, depth, z]);
        let b3 = pts.len();
        pts.push([hw, depth, z]);
        lines.push_cell(&[b2 as i64, b3 as i64]);
    }
    // Cross beams
    for &z in &[0.0, height] {
        let b0 = pts.len();
        pts.push([-hw, 0.0, z]);
        let b1 = pts.len();
        pts.push([-hw, depth, z]);
        lines.push_cell(&[b0 as i64, b1 as i64]);
        let b2 = pts.len();
        pts.push([hw, 0.0, z]);
        let b3 = pts.len();
        pts.push([hw, depth, z]);
        lines.push_cell(&[b2 as i64, b3 as i64]);
    }
    // Warp threads
    for i in 0..nw {
        let x = -hw * 0.8 + width * 0.8 * i as f64 / (nw - 1).max(1) as f64;
        let w0 = pts.len();
        pts.push([x, depth * 0.1, height * 0.2]);
        let w1 = pts.len();
        pts.push([x, depth * 0.9, height * 0.8]);
        lines.push_cell(&[w0 as i64, w1 as i64]);
    }
    // Cloth beam (roller at bottom front)
    let cb0 = pts.len();
    pts.push([-hw * 0.9, depth * 0.1, height * 0.15]);
    let cb1 = pts.len();
    pts.push([hw * 0.9, depth * 0.1, height * 0.15]);
    lines.push_cell(&[cb0 as i64, cb1 as i64]);
    // Warp beam (roller at top back)
    let wb0 = pts.len();
    pts.push([-hw * 0.9, depth * 0.9, height * 0.85]);
    let wb1 = pts.len();
    pts.push([hw * 0.9, depth * 0.9, height * 0.85]);
    lines.push_cell(&[wb0 as i64, wb1 as i64]);
    // Heddle frame
    let hf = pts.len();
    pts.push([-hw * 0.85, depth * 0.5, height * 0.5]);
    pts.push([hw * 0.85, depth * 0.5, height * 0.5]);
    pts.push([hw * 0.85, depth * 0.5, height * 0.7]);
    pts.push([-hw * 0.85, depth * 0.5, height * 0.7]);
    polys.push_cell(&[hf as i64, (hf + 1) as i64, (hf + 2) as i64, (hf + 3) as i64]);
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
    fn test_loom() {
        let m = loom(4.0, 5.0, 3.0, 10);
        assert!(m.points.len() > 30);
        assert!(m.lines.num_cells() > 15);
    }
}
