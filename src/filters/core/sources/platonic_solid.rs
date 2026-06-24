use crate::data::{CellArray, Points, PolyData};

/// Which platonic solid to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatonicSolidType {
    Tetrahedron,
    Octahedron,
    Icosahedron,
    Dodecahedron,
}

/// Generate a platonic solid centered at the origin with unit circumradius.
pub fn platonic_solid(solid_type: PlatonicSolidType) -> PolyData {
    match solid_type {
        PlatonicSolidType::Tetrahedron => make_tetrahedron(),
        PlatonicSolidType::Octahedron => make_octahedron(),
        PlatonicSolidType::Icosahedron => make_icosahedron(),
        PlatonicSolidType::Dodecahedron => make_dodecahedron(),
    }
}

fn build_poly_data(verts: &[[f64; 3]], faces: &[Vec<i64>]) -> PolyData {
    let mut points = Points::new();
    let mut polys = CellArray::new();

    for &v in verts {
        points.push(v);
    }
    for face in faces {
        polys.push_cell(face);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

fn make_tetrahedron() -> PolyData {
    let a = 1.0 / 3.0_f64.sqrt();
    let verts = [[a, a, a], [a, -a, -a], [-a, a, -a], [-a, -a, a]];
    let faces = vec![vec![0, 1, 2], vec![0, 3, 1], vec![0, 2, 3], vec![1, 3, 2]];
    build_poly_data(&verts, &faces)
}

fn make_octahedron() -> PolyData {
    let verts = [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];
    let faces = vec![
        vec![0, 2, 4],
        vec![0, 4, 3],
        vec![0, 3, 5],
        vec![0, 5, 2],
        vec![1, 4, 2],
        vec![1, 3, 4],
        vec![1, 5, 3],
        vec![1, 2, 5],
    ];
    build_poly_data(&verts, &faces)
}

fn make_icosahedron() -> PolyData {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let len = (1.0 + phi * phi).sqrt();
    let a = 1.0 / len;
    let b = phi / len;

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
    let faces = vec![
        vec![0, 11, 5],
        vec![0, 5, 1],
        vec![0, 1, 7],
        vec![0, 7, 10],
        vec![0, 10, 11],
        vec![1, 5, 9],
        vec![5, 11, 4],
        vec![11, 10, 2],
        vec![10, 7, 6],
        vec![7, 1, 8],
        vec![3, 9, 4],
        vec![3, 4, 2],
        vec![3, 2, 6],
        vec![3, 6, 8],
        vec![3, 8, 9],
        vec![4, 9, 5],
        vec![2, 4, 11],
        vec![6, 2, 10],
        vec![8, 6, 7],
        vec![9, 8, 1],
    ];
    build_poly_data(&verts, &faces)
}

fn make_dodecahedron() -> PolyData {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let a = 1.0 / (3.0_f64.sqrt());
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
    let faces = vec![
        vec![0, 16, 2, 10, 8],
        vec![0, 8, 4, 14, 12],
        vec![16, 17, 1, 12, 0],
        vec![1, 9, 11, 3, 17],
        vec![1, 12, 14, 5, 9],
        vec![2, 13, 15, 6, 10],
        vec![13, 3, 17, 16, 2],
        vec![3, 11, 7, 15, 13],
        vec![4, 8, 10, 6, 18],
        vec![14, 5, 19, 18, 4],
        vec![5, 19, 7, 11, 9],
        vec![15, 7, 19, 18, 6],
    ];
    build_poly_data(&verts, &faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tetrahedron() {
        let pd = platonic_solid(PlatonicSolidType::Tetrahedron);
        assert_eq!(pd.points.len(), 4);
        assert_eq!(pd.polys.num_cells(), 4);
    }

    #[test]
    fn octahedron() {
        let pd = platonic_solid(PlatonicSolidType::Octahedron);
        assert_eq!(pd.points.len(), 6);
        assert_eq!(pd.polys.num_cells(), 8);
    }

    #[test]
    fn icosahedron() {
        let pd = platonic_solid(PlatonicSolidType::Icosahedron);
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 20);
    }

    #[test]
    fn dodecahedron() {
        let pd = platonic_solid(PlatonicSolidType::Dodecahedron);
        assert_eq!(pd.points.len(), 20);
        assert_eq!(pd.polys.num_cells(), 12);
    }
}
