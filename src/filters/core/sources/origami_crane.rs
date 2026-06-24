//! Origami crane outline (simplified wireframe).
use crate::data::{CellArray, Points, PolyData};

pub fn origami_crane(size: f64) -> PolyData {
    let s = size;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Body (diamond shape)
    let b0 = pts.len();
    pts.push([0.0, 0.0, 0.0]); // center
    let b1 = pts.len();
    pts.push([s * 0.4, 0.0, s * 0.2]); // right
    let b2 = pts.len();
    pts.push([0.0, 0.0, s * 0.5]); // front
    let b3 = pts.len();
    pts.push([-s * 0.4, 0.0, s * 0.2]); // left
    let b4 = pts.len();
    pts.push([0.0, 0.0, -s * 0.1]); // back
    polys.push_cell(&[b0 as i64, b1 as i64, b2 as i64]);
    polys.push_cell(&[b0 as i64, b2 as i64, b3 as i64]);
    polys.push_cell(&[b0 as i64, b3 as i64, b4 as i64]);
    polys.push_cell(&[b0 as i64, b4 as i64, b1 as i64]);
    // Wings (triangles extending from sides)
    let w1 = pts.len();
    pts.push([s * 1.0, s * 0.05, s * 0.15]);
    let w2 = pts.len();
    pts.push([-s * 1.0, s * 0.05, s * 0.15]);
    polys.push_cell(&[b1 as i64, w1 as i64, b2 as i64]);
    polys.push_cell(&[b3 as i64, b2 as i64, w2 as i64]);
    // Neck and head
    let n1 = pts.len();
    pts.push([0.0, s * 0.02, s * 0.7]);
    let head = pts.len();
    pts.push([0.0, s * 0.01, s * 0.85]);
    polys.push_cell(&[b2 as i64, n1 as i64, head as i64]);
    // Tail
    let tail = pts.len();
    pts.push([0.0, s * 0.03, -s * 0.4]);
    polys.push_cell(&[b4 as i64, tail as i64, b0 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_crane() {
        let m = origami_crane(5.0);
        assert!(m.points.len() >= 9);
        assert!(m.polys.num_cells() >= 6);
    }
}
