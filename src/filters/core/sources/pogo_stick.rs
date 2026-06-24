//! Pogo stick with spring and footpegs.
use crate::data::{CellArray, Points, PolyData};

pub fn pogo_stick(height: f64, spring_coils: usize) -> PolyData {
    let nc = spring_coils.max(5);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Main shaft
    let s0 = pts.len();
    pts.push([0.0, 0.0, height]);
    let s1 = pts.len();
    pts.push([0.0, 0.0, height * 0.4]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Handle grips
    let hw = height * 0.1;
    let h0 = pts.len();
    pts.push([-hw, 0.0, height]);
    let h1 = pts.len();
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[h0 as i64, h1 as i64]);
    // Foot pegs
    for &sx in &[-1.0f64, 1.0] {
        let fp = pts.len();
        pts.push([sx * hw * 0.8, 0.0, height * 0.5]);
        lines.push_cell(&[s1 as i64, fp as i64]);
    }
    // Spring section
    let spring_r = height * 0.03;
    let spring_h = height * 0.35;
    let n_pts_spring = nc * 8;
    let sp_base = pts.len();
    for i in 0..=n_pts_spring {
        let t = i as f64 / n_pts_spring as f64;
        let z = height * 0.05 + spring_h * t;
        let a = 2.0 * std::f64::consts::PI * nc as f64 * t;
        pts.push([spring_r * a.cos(), spring_r * a.sin(), z]);
    }
    for i in 0..n_pts_spring {
        lines.push_cell(&[(sp_base + i) as i64, (sp_base + i + 1) as i64]);
    }
    // Bottom shaft + foot pad
    let b0 = pts.len();
    pts.push([0.0, 0.0, height * 0.05]);
    let b1 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    lines.push_cell(&[b0 as i64, b1 as i64]);
    // Foot pad (small circle)
    let pad_r = height * 0.04;
    let pb = pts.len();
    for j in 0..8 {
        let a = 2.0 * std::f64::consts::PI * j as f64 / 8.0;
        pts.push([pad_r * a.cos(), pad_r * a.sin(), 0.0]);
    }
    for j in 0..8 {
        lines.push_cell(&[(pb + j) as i64, (pb + (j + 1) % 8) as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pogo() {
        let m = pogo_stick(5.0, 8);
        assert!(m.points.len() > 50);
        assert!(m.lines.num_cells() > 50);
    }
}
