//! Counterweight (heavy block on a hinge).
use crate::data::{CellArray, Points, PolyData};

pub fn counterweight(size: f64, arm_length: f64) -> PolyData {
    let hs = size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pivot point
    let pivot = pts.len();
    pts.push([0.0, 0.0, arm_length]);
    // Arm
    let arm_end = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    lines.push_cell(&[pivot as i64, arm_end as i64]);
    // Weight block
    let bb = pts.len();
    pts.push([-hs, -hs, -size]);
    pts.push([hs, -hs, -size]);
    pts.push([hs, hs, -size]);
    pts.push([-hs, hs, -size]);
    pts.push([-hs, -hs, 0.0]);
    pts.push([hs, -hs, 0.0]);
    pts.push([hs, hs, 0.0]);
    pts.push([-hs, hs, 0.0]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]); // bottom
    polys.push_cell(&[(bb + 4) as i64, (bb + 5) as i64, (bb + 1) as i64, bb as i64]); // front
    polys.push_cell(&[
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 2) as i64,
        (bb + 1) as i64,
    ]); // right
    polys.push_cell(&[
        (bb + 6) as i64,
        (bb + 7) as i64,
        (bb + 3) as i64,
        (bb + 2) as i64,
    ]); // back
    polys.push_cell(&[(bb + 7) as i64, (bb + 4) as i64, bb as i64, (bb + 3) as i64]); // left
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]); // top
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
    fn test_cw() {
        let m = counterweight(2.0, 5.0);
        assert!(m.points.len() >= 10);
        assert!(m.polys.num_cells() == 6);
    }
}
