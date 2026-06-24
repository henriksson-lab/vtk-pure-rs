//! Diamond (octahedron) and bipyramid geometry sources.

use crate::data::{CellArray, Points, PolyData};

/// Create a diamond (regular octahedron).
pub fn diamond(radius: f64) -> PolyData {
    let r = radius;
    let verts = [
        [0.0, 0.0, r],  // top
        [r, 0.0, 0.0],  // +x
        [0.0, r, 0.0],  // +y
        [-r, 0.0, 0.0], // -x
        [0.0, -r, 0.0], // -y
        [0.0, 0.0, -r], // bottom
    ];
    let faces: Vec<[usize; 3]> = vec![
        [0, 1, 2],
        [0, 2, 3],
        [0, 3, 4],
        [0, 4, 1],
        [5, 2, 1],
        [5, 3, 2],
        [5, 4, 3],
        [5, 1, 4],
    ];
    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

/// Create an N-sided bipyramid (two pyramids joined at the base).
pub fn bipyramid(sides: usize, radius: f64, height: f64) -> PolyData {
    let sides = sides.max(3);
    let half_h = height * 0.5;
    let mut pts = Points::<f64>::new();
    pts.push([0.0, 0.0, half_h]); // top
    pts.push([0.0, 0.0, -half_h]); // bottom
    for i in 0..sides {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / sides as f64;
        pts.push([radius * angle.cos(), radius * angle.sin(), 0.0]);
    }
    let mut polys = CellArray::new();
    for i in 0..sides {
        let a = (i + 2) as i64;
        let b = ((i + 1) % sides + 2) as i64;
        polys.push_cell(&[0, a, b]); // top pyramid
        polys.push_cell(&[1, b, a]); // bottom pyramid
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_diamond() {
        let d = diamond(1.0);
        assert_eq!(d.points.len(), 6);
        assert_eq!(d.polys.num_cells(), 8);
    }
    #[test]
    fn test_bipyramid() {
        let b = bipyramid(5, 1.0, 2.0);
        assert_eq!(b.points.len(), 7); // 2 apex + 5 base
        assert_eq!(b.polys.num_cells(), 10); // 5 top + 5 bottom
    }
    #[test]
    fn test_bipyramid_triangle() {
        let b = bipyramid(3, 1.0, 1.0);
        assert_eq!(b.points.len(), 5);
        assert_eq!(b.polys.num_cells(), 6);
    }
}
