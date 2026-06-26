//! Fibonacci/golden spiral as a 3D curve.
use crate::data::{CellArray, Points, PolyData};

pub fn fibonacci_spiral(turns: f64, n_pts: usize) -> PolyData {
    let n = n_pts.max(20);
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for i in 0..=n {
        let t = i as f64 / n as f64 * turns;
        let angle = 2.0 * std::f64::consts::PI * t;
        let r = phi.powf(angle * 2.0 / std::f64::consts::PI);
        pts.push([r * angle.cos(), r * angle.sin(), t * 0.5]);
    }
    for i in 0..n {
        lines.push_cell(&[i as i64, (i + 1) as i64]);
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
    fn test_fib_spiral() {
        let m = fibonacci_spiral(3.0, 50);
        assert_eq!(m.points.len(), 51);
        assert_eq!(m.lines.num_cells(), 50);
    }

    #[test]
    fn radius_grows_by_phi_per_quarter_turn() {
        let m = fibonacci_spiral(1.0, 4);
        assert!(
            (m.points.get(1)[0].hypot(m.points.get(1)[1]) - (1.0 + 5.0f64.sqrt()) / 2.0).abs()
                < 1e-12
        );
    }
}
