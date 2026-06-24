//! Cross-polytope (hyperoctahedron) and orthoplex projections.
use crate::data::{CellArray, Points, PolyData};
pub fn cross_polytope_3d(radius: f64) -> PolyData {
    // Regular octahedron (3D cross-polytope)
    let r = radius;
    let verts = [
        [r, 0.0, 0.0],
        [-r, 0.0, 0.0],
        [0.0, r, 0.0],
        [0.0, -r, 0.0],
        [0.0, 0.0, r],
        [0.0, 0.0, -r],
    ];
    let faces = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [0, 5, 2],
        [2, 5, 1],
        [1, 5, 3],
        [3, 5, 0],
    ];
    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
    }
    let mut r2 = PolyData::new();
    r2.points = pts;
    r2.polys = polys;
    r2
}
pub fn tesseract_projection(radius: f64) -> PolyData {
    // 4D hypercube projected to 3D via perspective
    let r = radius;
    let d = 3.0; // projection distance
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // 16 vertices of 4D hypercube
    for w in &[-1.0f64, 1.0] {
        for z in &[-1.0f64, 1.0] {
            for y in &[-1.0f64, 1.0] {
                for x in &[-1.0f64, 1.0] {
                    let scale = r * d / (d + w * 0.5);
                    pts.push([x * scale, y * scale, z * scale]);
                }
            }
        }
    }
    // 32 edges: vertices differ by exactly one coordinate
    for i in 0..16 {
        for j in i + 1..16 {
            let diff = (0..4).filter(|&b| ((i >> b) & 1) != ((j >> b) & 1)).count();
            if diff == 1 {
                lines.push_cell(&[i as i64, j as i64]);
            }
        }
    }
    let mut r2 = PolyData::new();
    r2.points = pts;
    r2.lines = lines;
    r2
}
pub fn pentatope_projection(radius: f64) -> PolyData {
    // 5-cell (4D simplex) projected to 3D
    let r = radius;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // 5 vertices of regular 4-simplex projected
    let verts = [
        [r, r, r],
        [r, -r, -r],
        [-r, r, -r],
        [-r, -r, r],
        [0.0, 0.0, 0.0],
    ];
    for v in &verts {
        pts.push(*v);
    }
    // All 10 edges
    for i in 0..5 {
        for j in i + 1..5 {
            lines.push_cell(&[i as i64, j as i64]);
        }
    }
    // All 10 triangular faces
    for i in 0..5 {
        for j in i + 1..5 {
            for k in j + 1..5 {
                polys.push_cell(&[i as i64, j as i64, k as i64]);
            }
        }
    }
    let mut r2 = PolyData::new();
    r2.points = pts;
    r2.lines = lines;
    r2.polys = polys;
    r2
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_octahedron() {
        let o = cross_polytope_3d(1.0);
        assert_eq!(o.polys.num_cells(), 8);
    }
    #[test]
    fn test_tesseract() {
        let t = tesseract_projection(1.0);
        assert_eq!(t.points.len(), 16);
        assert_eq!(t.lines.num_cells(), 32);
    }
    #[test]
    fn test_pentatope() {
        let p = pentatope_projection(1.0);
        assert_eq!(p.points.len(), 5);
        assert_eq!(p.lines.num_cells(), 10);
    }
}
