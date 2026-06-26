//! Regular polyhedra (Platonic solids) sources.

use crate::data::{CellArray, Points, PolyData};

/// Create an icosahedron with given radius.
pub fn icosahedron(radius: f64) -> PolyData {
    let c = 0.5;
    let d = 0.30901699;
    let verts = [
        [0.0, d, -c],
        [0.0, d, c],
        [0.0, -d, c],
        [-d, c, 0.0],
        [-d, -c, 0.0],
        [d, c, 0.0],
        [d, -c, 0.0],
        [0.0, -d, -c],
        [c, 0.0, d],
        [-c, 0.0, d],
        [-c, 0.0, -d],
        [c, 0.0, -d],
    ];
    let faces: [[usize; 3]; 20] = [
        [0, 3, 5],
        [1, 5, 3],
        [1, 9, 2],
        [1, 2, 8],
        [0, 11, 7],
        [0, 7, 10],
        [2, 4, 6],
        [7, 6, 4],
        [3, 10, 9],
        [4, 9, 10],
        [5, 8, 11],
        [6, 11, 8],
        [1, 3, 9],
        [1, 8, 5],
        [0, 10, 3],
        [0, 5, 11],
        [7, 4, 10],
        [7, 11, 6],
        [2, 9, 4],
        [2, 6, 8],
    ];
    build_polyhedron(&verts, &faces, radius / 0.58778524999243)
}

/// Create a dodecahedron with given radius.
pub fn dodecahedron(radius: f64) -> PolyData {
    let a = 0.61803398875;
    let b = 0.381966011250;
    let verts = [
        [b, 0.0, 1.0],
        [-b, 0.0, 1.0],
        [b, 0.0, -1.0],
        [-b, 0.0, -1.0],
        [0.0, 1.0, -b],
        [0.0, 1.0, b],
        [0.0, -1.0, -b],
        [0.0, -1.0, b],
        [1.0, b, 0.0],
        [1.0, -b, 0.0],
        [-1.0, b, 0.0],
        [-1.0, -b, 0.0],
        [-a, a, a],
        [a, -a, a],
        [-a, -a, -a],
        [a, a, -a],
        [a, a, a],
        [-a, a, -a],
        [-a, -a, a],
        [a, -a, -a],
    ];
    let faces: [[usize; 5]; 12] = [
        [0, 16, 5, 12, 1],
        [1, 18, 7, 13, 0],
        [2, 19, 6, 14, 3],
        [3, 17, 4, 15, 2],
        [4, 5, 16, 8, 15],
        [5, 4, 17, 10, 12],
        [6, 7, 18, 11, 14],
        [7, 6, 19, 9, 13],
        [8, 16, 0, 13, 9],
        [9, 19, 2, 15, 8],
        [10, 17, 3, 14, 11],
        [11, 18, 1, 12, 10],
    ];
    let mut pts = Points::<f64>::new();
    let scale = radius / 1.070466269319;
    for v in &verts {
        pts.push([scale * v[0], scale * v[1], scale * v[2]]);
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
    let r = radius / 2.0f64.sqrt();
    let verts = [
        [-r, -r, 0.0],
        [r, -r, 0.0],
        [r, r, 0.0],
        [-r, r, 0.0],
        [0.0, 0.0, -radius],
        [0.0, 0.0, radius],
    ];
    let faces = [
        [4, 1, 0],
        [4, 2, 1],
        [4, 3, 2],
        [4, 0, 3],
        [0, 1, 5],
        [1, 2, 5],
        [2, 3, 5],
        [3, 0, 5],
    ];
    build_polyhedron(&verts, &faces, 1.0)
}

fn build_polyhedron(verts: &[[f64; 3]], faces: &[[usize; 3]], scale: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    for v in verts {
        pts.push([scale * v[0], scale * v[1], scale * v[2]]);
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
