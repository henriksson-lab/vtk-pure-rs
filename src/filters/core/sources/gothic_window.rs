//! Gothic pointed arch window frame.
use crate::data::{CellArray, Points, PolyData};

pub fn gothic_window(width: f64, height: f64, n_arch: usize) -> PolyData {
    let na = n_arch.max(8);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Rectangular frame
    let f0 = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    let f1 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    let f2 = pts.len();
    pts.push([hw, 0.0, height]);
    let f3 = pts.len();
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    lines.push_cell(&[f0 as i64, f3 as i64]);
    lines.push_cell(&[f1 as i64, f2 as i64]);
    // Pointed arch at top
    let arch_base = pts.len();
    let r = width; // radius of each arc = full width
    for i in 0..=na {
        let t = i as f64 / na as f64;
        let angle = std::f64::consts::PI / 3.0 * t; // 60 degree arc from left
        let x = -hw + r * angle.sin();
        let z = height + r * (angle.cos() - 1.0) + r * (std::f64::consts::PI / 3.0).cos();
        pts.push([x.min(0.0), 0.0, z]);
    }
    for i in 0..na {
        lines.push_cell(&[(arch_base + i) as i64, (arch_base + i + 1) as i64]);
    }
    let arch_right = pts.len();
    for i in 0..=na {
        let t = i as f64 / na as f64;
        let angle = std::f64::consts::PI / 3.0 * (1.0 - t);
        let x = hw - r * angle.sin();
        let z = height + r * (angle.cos() - 1.0) + r * (std::f64::consts::PI / 3.0).cos();
        pts.push([x.max(0.0), 0.0, z]);
    }
    for i in 0..na {
        lines.push_cell(&[(arch_right + i) as i64, (arch_right + i + 1) as i64]);
    }
    // Central mullion
    let m0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let m1 = pts.len();
    pts.push([0.0, 0.0, height]);
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
    }
}
