//! Single nucleotide (sugar-phosphate backbone + base).
use crate::data::{CellArray, Points, PolyData};

pub fn nucleotide(size: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Phosphate group (small triangle)
    let p0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let p1 = pts.len();
    pts.push([size * 0.2, 0.0, size * 0.1]);
    let p2 = pts.len();
    pts.push([-size * 0.1, 0.0, size * 0.15]);
    lines.push_cell(&[p0 as i64, p1 as i64]);
    lines.push_cell(&[p1 as i64, p2 as i64]);
    lines.push_cell(&[p2 as i64, p0 as i64]);
    // Sugar (pentagon)
    let sc = [size * 0.3, 0.0, size * 0.3];
    let sr = size * 0.15;
    let sb = pts.len();
    for j in 0..5 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 5.0;
        pts.push([sc[0] + sr * a.cos(), 0.0, sc[2] + sr * a.sin()]);
    }
    for j in 0..5 {
        lines.push_cell(&[(sb + j) as i64, (sb + (j + 1) % 5) as i64]);
    }
    // Connect phosphate to sugar
    lines.push_cell(&[p1 as i64, sb as i64]);
    // Base (hexagon)
    let bc = [size * 0.3, 0.0, size * 0.65];
    let br = size * 0.2;
    let bb = pts.len();
    for j in 0..6 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
        pts.push([bc[0] + br * a.cos(), 0.0, bc[2] + br * a.sin()]);
    }
    for j in 0..6 {
        lines.push_cell(&[(bb + j) as i64, (bb + (j + 1) % 6) as i64]);
    }
    // Connect sugar to base
    lines.push_cell(&[(sb + 2) as i64, bb as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nucleotide() {
        let m = nucleotide(1.0);
        assert!(m.points.len() > 10);
        assert!(m.lines.num_cells() > 10);
    }
}
