//! Gothic pointed arch window frame.
use crate::data::{CellArray, Points, PolyData};

pub fn gothic_window(width: f64, height: f64, n_arch: usize) -> PolyData {
    let na = n_arch.max(8);
    let hw = width.abs() / 2.0;
    let spring_z = height;
    let arch_r = width.abs();
    let arch_rise = (arch_r * arch_r - hw * hw).max(0.0).sqrt();
    let apex_z = spring_z + arch_rise;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Rectangular frame
    let f0 = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    let f1 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    let f2 = pts.len();
    pts.push([hw, 0.0, spring_z]);
    let f3 = pts.len();
    pts.push([-hw, 0.0, spring_z]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    lines.push_cell(&[f0 as i64, f3 as i64]);
    lines.push_cell(&[f1 as i64, f2 as i64]);
    // Pointed arch at top
    let arch_base = pts.len();
    for i in 0..=na {
        let t = i as f64 / na as f64;
        let angle = std::f64::consts::PI - t * std::f64::consts::PI / 3.0;
        let x = hw + arch_r * angle.cos();
        let z = spring_z + arch_r * angle.sin();
        pts.push([x, 0.0, z]);
    }
    for i in 0..na {
        lines.push_cell(&[(arch_base + i) as i64, (arch_base + i + 1) as i64]);
    }
    let arch_right = pts.len();
    for i in 0..=na {
        let t = i as f64 / na as f64;
        let angle = std::f64::consts::PI / 3.0 * (1.0 - t);
        let x = -hw + arch_r * angle.cos();
        let z = spring_z + arch_r * angle.sin();
        pts.push([x, 0.0, z]);
    }
    for i in 0..na {
        lines.push_cell(&[(arch_right + i) as i64, (arch_right + i + 1) as i64]);
    }
    // Central mullion
    let m0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let m1 = pts.len();
    pts.push([0.0, 0.0, apex_z]);
    lines.push_cell(&[m0 as i64, m1 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gothic() {
        let m = gothic_window(2.0, 4.0, 12);
        assert!(m.points.len() > 20);
        assert!(m.lines.num_cells() > 5);
        assert!((m.points.get(4)[0] + 1.0).abs() < 1e-12);
        assert!((m.points.get(4)[2] - 4.0).abs() < 1e-12);
        let apex = m.points.get(4 + 12);
        assert!(apex[0].abs() < 1e-12);
        assert!(apex[2] > 4.0);
    }
}
