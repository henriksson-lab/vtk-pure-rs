//! Three-axis gimbal (nested rings for stabilization).
use crate::data::{CellArray, Points, PolyData};

pub fn gyroscope_gimbal(radius: f64, na: usize) -> PolyData {
    let na = na.max(20);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Three orthogonal rings with increasing radii
    let rings: Vec<([f64; 3], f64)> = vec![
        ([1.0, 0.0, 0.0], radius * 0.6), // inner: rotation around X
        ([0.0, 1.0, 0.0], radius * 0.8), // middle: rotation around Y
        ([0.0, 0.0, 1.0], radius),       // outer: rotation around Z
    ];
    for (axis, r) in &rings {
        let up = if axis[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = [
            axis[1] * up[2] - axis[2] * up[1],
            axis[2] * up[0] - axis[0] * up[2],
            axis[0] * up[1] - axis[1] * up[0],
        ];
        let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = [u[0] / ul, u[1] / ul, u[2] / ul];
        let v = [
            axis[1] * u[2] - axis[2] * u[1],
            axis[2] * u[0] - axis[0] * u[2],
            axis[0] * u[1] - axis[1] * u[0],
        ];
        let rb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                r * (u[0] * a.cos() + v[0] * a.sin()),
                r * (u[1] * a.cos() + v[1] * a.sin()),
                r * (u[2] * a.cos() + v[2] * a.sin()),
            ]);
        }
        for j in 0..na {
            lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
        }
    }
    // Pivot pins connecting rings at their geometric intersections.
    for &(p_inner, p_outer) in &[
        ([0.0, 0.0, radius * 0.6], [0.0, 0.0, radius * 0.8]),
        ([0.0, 0.0, -radius * 0.6], [0.0, 0.0, -radius * 0.8]),
        ([radius * 0.8, 0.0, 0.0], [radius, 0.0, 0.0]),
        ([-radius * 0.8, 0.0, 0.0], [-radius, 0.0, 0.0]),
    ] {
        let p0 = pts.len();
        pts.push(p_inner);
        let p1 = pts.len();
        pts.push(p_outer);
        lines.push_cell(&[p0 as i64, p1 as i64]);
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
    fn test_gimbal() {
        let m = gyroscope_gimbal(3.0, 24);
        assert!(m.points.len() > 70);
        assert!(m.lines.num_cells() > 60);
    }
}
