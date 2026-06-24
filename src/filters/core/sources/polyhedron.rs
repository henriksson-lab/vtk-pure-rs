//! Regular polyhedra (Platonic solids) sources.

use crate::data::{CellArray, Points, PolyData};

/// Create an icosahedron with given radius.
pub fn icosahedron(radius: f64) -> PolyData {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = radius / (1.0 + phi * phi).sqrt();
    let b = a * phi;
    let verts = [
        [-a, b, 0.0],
        [a, b, 0.0],
        [-a, -b, 0.0],
        [a, -b, 0.0],
        [0.0, -a, b],
        [0.0, a, b],
        [0.0, -a, -b],
        [0.0, a, -b],
        [b, 0.0, -a],
        [b, 0.0, a],
        [-b, 0.0, -a],
        [-b, 0.0, a],
    ];
    let faces: [[usize; 3]; 20] = [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    build_polyhedron(&verts, &faces)
}

/// Create a dodecahedron with given radius.
pub fn dodecahedron(radius: f64) -> PolyData {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = radius / 3.0f64.sqrt();
    let b = a / phi;
    let c = a * phi;
    let verts = [
        [a, a, a],
        [a, a, -a],
        [a, -a, a],
        [a, -a, -a],
        [-a, a, a],
        [-a, a, -a],
        [-a, -a, a],
        [-a, -a, -a],
        [0.0, b, c],
        [0.0, b, -c],
        [0.0, -b, c],
        [0.0, -b, -c],
        [b, c, 0.0],
        [b, -c, 0.0],
        [-b, c, 0.0],
        [-b, -c, 0.0],
        [c, 0.0, b],
        [c, 0.0, -b],
        [-c, 0.0, b],
        [-c, 0.0, -b],
    ];
    // Dodecahedron has 12 pentagonal faces
    let faces: [[usize; 5]; 12] = [
        [0, 16, 2, 10, 8],
        [0, 8, 4, 14, 12],
        [16, 17, 1, 12, 0],
        [1, 9, 11, 3, 17],
        [1, 12, 14, 5, 9],
        [2, 13, 15, 6, 10],
        [13, 3, 17, 16, 2],
        [3, 11, 7, 15, 13],
        [4, 8, 10, 6, 18],
        [14, 5, 19, 18, 4],
        [5, 9, 11, 7, 19],
        [15, 7, 19, 18, 6],
    ];
    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in &faces {
        polys.push_cell(&f.iter().map(|&i| i as i64).collect::<Vec<_>>());
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create an octahedron with given radius.
pub fn octahedron(radius: f64) -> PolyData {
    let r = radius;
    let verts = [
        [r, 0.0, 0.0],
        [0.0, r, 0.0],
        [0.0, 0.0, r],
        [-r, 0.0, 0.0],
        [0.0, -r, 0.0],
        [0.0, 0.0, -r],
    ];
    let faces = [
        [0, 1, 2],
        [1, 3, 2],
        [3, 4, 2],
        [4, 0, 2],
        [1, 0, 5],
        [3, 1, 5],
        [4, 3, 5],
        [0, 4, 5],
    ];
    build_polyhedron(&verts, &faces)
}

fn build_polyhedron(verts: &[[f64; 3]], faces: &[[usize; 3]]) -> PolyData {
    let mut pts = Points::<f64>::new();
    for v in verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
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
    fn test_icosahedron() {
        let i = icosahedron(1.0);
        assert_eq!(i.points.len(), 12);
        assert_eq!(i.polys.num_cells(), 20);
    }
    #[test]
    fn test_dodecahedron() {
        let d = dodecahedron(1.0);
        assert_eq!(d.points.len(), 20);
        assert_eq!(d.polys.num_cells(), 12);
    }
    #[test]
    fn test_octahedron() {
        let o = octahedron(1.0);
        assert_eq!(o.points.len(), 6);
        assert_eq!(o.polys.num_cells(), 8);
    }
}
