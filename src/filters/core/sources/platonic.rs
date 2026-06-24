//! All five Platonic solids with consistent API.

use crate::data::{CellArray, Points, PolyData};

/// Platonic solid type.
pub enum PlatonicSolid {
    Tetrahedron,
    Cube,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

/// Create a Platonic solid centered at origin with given circumradius.
pub fn platonic(solid: PlatonicSolid, radius: f64) -> PolyData {
    match solid {
        PlatonicSolid::Tetrahedron => make_tetrahedron(radius),
        PlatonicSolid::Cube => make_cube(radius),
        PlatonicSolid::Octahedron => make_octahedron(radius),
        PlatonicSolid::Dodecahedron => make_dodecahedron(radius),
        PlatonicSolid::Icosahedron => make_icosahedron(radius),
    }
}

fn make_tetrahedron(r: f64) -> PolyData {
    let a = r * 2.0 / 3.0f64.sqrt();
    let verts = [[a, a, a], [a, -a, -a], [-a, a, -a], [-a, -a, a]];
    let scale = r / (a * a * 3.0).sqrt();
    let verts: Vec<[f64; 3]> = verts
        .iter()
        .map(|v| [v[0] * scale, v[1] * scale, v[2] * scale])
        .collect();
    build(&verts, &[[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]])
}

fn make_cube(r: f64) -> PolyData {
    let a = r / 3.0f64.sqrt();
    let v = [
        [-a, -a, -a],
        [a, -a, -a],
        [a, a, -a],
        [-a, a, -a],
        [-a, -a, a],
        [a, -a, a],
        [a, a, a],
        [-a, a, a],
    ];
    build_quads(
        &v,
        &[
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ],
    )
}

fn make_octahedron(r: f64) -> PolyData {
    let v = [
        [r, 0.0, 0.0],
        [0.0, r, 0.0],
        [0.0, 0.0, r],
        [-r, 0.0, 0.0],
        [0.0, -r, 0.0],
        [0.0, 0.0, -r],
    ];
    build(
        &v,
        &[
            [0, 1, 2],
            [1, 3, 2],
            [3, 4, 2],
            [4, 0, 2],
            [1, 0, 5],
            [3, 1, 5],
            [4, 3, 5],
            [0, 4, 5],
        ],
    )
}

fn make_dodecahedron(r: f64) -> PolyData {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = r / 3.0f64.sqrt();
    let b = a / phi;
    let c = a * phi;
    let v = [
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
    let faces: &[[usize; 5]] = &[
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
    for vv in &v {
        pts.push(*vv);
    }
    let mut polys = CellArray::new();
    for f in faces {
        polys.push_cell(&f.iter().map(|&i| i as i64).collect::<Vec<_>>());
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn make_icosahedron(r: f64) -> PolyData {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = r / (1.0 + phi * phi).sqrt();
    let b = a * phi;
    let v = [
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
    build(
        &v,
        &[
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
        ],
    )
}

fn build(verts: &[[f64; 3]], faces: &[[usize; 3]]) -> PolyData {
    let mut pts = Points::<f64>::new();
    for v in verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

fn build_quads(verts: &[[f64; 3]], faces: &[[usize; 4]]) -> PolyData {
    let mut pts = Points::<f64>::new();
    for v in verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64, f[3] as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_all() {
        let cases = [
            (PlatonicSolid::Tetrahedron, 4, 4),
            (PlatonicSolid::Cube, 8, 6),
            (PlatonicSolid::Octahedron, 6, 8),
            (PlatonicSolid::Dodecahedron, 20, 12),
            (PlatonicSolid::Icosahedron, 12, 20),
        ];
        for (solid, nv, nf) in cases {
            let p = platonic(solid, 1.0);
            assert_eq!(p.points.len(), nv);
            assert_eq!(p.polys.num_cells(), nf);
        }
    }
}
