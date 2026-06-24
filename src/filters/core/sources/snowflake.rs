//! Koch snowflake fractal curve.
use crate::data::{CellArray, Points, PolyData};

pub fn snowflake(size: f64, iterations: usize) -> PolyData {
    // Start with equilateral triangle
    let h = size * 3.0f64.sqrt() / 2.0;
    let mut curve: Vec<[f64; 2]> = vec![
        [-size / 2.0, -h / 3.0],
        [size / 2.0, -h / 3.0],
        [0.0, 2.0 * h / 3.0],
        [-size / 2.0, -h / 3.0], // close the loop
    ];
    for _ in 0..iterations.min(5) {
        let mut next = Vec::new();
        for i in 0..curve.len() - 1 {
            let (ax, ay) = (curve[i][0], curve[i][1]);
            let (bx, by) = (curve[i + 1][0], curve[i + 1][1]);
            let dx = bx - ax;
            let dy = by - ay;
            let p1 = [ax + dx / 3.0, ay + dy / 3.0];
            let p2 = [
                ax + dx / 2.0 - dy * 3.0f64.sqrt() / 6.0,
                ay + dy / 2.0 + dx * 3.0f64.sqrt() / 6.0,
            ];
            let p3 = [ax + 2.0 * dx / 3.0, ay + 2.0 * dy / 3.0];
            next.push(curve[i]);
            next.push(p1);
            next.push(p2);
            next.push(p3);
        }
        next.push(*curve.last().unwrap());
        curve = next;
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for p in &curve {
        pts.push([p[0], p[1], 0.0]);
    }
    for i in 0..curve.len() - 1 {
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
    fn test_snowflake() {
        let m = snowflake(10.0, 3);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 100);
    }
}
