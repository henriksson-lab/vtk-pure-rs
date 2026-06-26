use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Which platonic solid to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatonicSolidType {
    Tetrahedron,
    Cube,
    Octahedron,
    Icosahedron,
    Dodecahedron,
}

/// Generate a platonic solid centered at the origin with unit circumradius.
pub fn platonic_solid(solid_type: PlatonicSolidType) -> PolyData {
    let (solid_points, solid_verts, cell_size, solid_scale) = match solid_type {
        PlatonicSolidType::Tetrahedron => {
            (&TETRA_POINTS[..], &TETRA_VERTS[..], 3, 1.0 / 3.0f64.sqrt())
        }
        PlatonicSolidType::Cube => (&CUBE_POINTS[..], &CUBE_VERTS[..], 4, 1.0 / 3.0f64.sqrt()),
        PlatonicSolidType::Octahedron => (&OCT_POINTS[..], &OCT_VERTS[..], 3, 1.0 / 2.0f64.sqrt()),
        PlatonicSolidType::Icosahedron => (
            &ICOSA_POINTS[..],
            &ICOSA_VERTS[..],
            3,
            1.0 / 0.58778524999243,
        ),
        PlatonicSolidType::Dodecahedron => {
            (&DODE_POINTS[..], &DODE_VERTS[..], 5, 1.0 / 1.070466269319)
        }
    };

    build_poly_data(solid_points, solid_verts, cell_size, solid_scale)
}

const TETRA_POINTS: [[f64; 3]; 4] = [
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [1.0, -1.0, -1.0],
    [-1.0, -1.0, 1.0],
];
const TETRA_VERTS: [i64; 12] = [0, 2, 1, 1, 2, 3, 0, 3, 2, 0, 1, 3];

const CUBE_POINTS: [[f64; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
];
const CUBE_VERTS: [i64; 24] = [
    0, 1, 5, 4, 0, 4, 7, 3, 4, 5, 6, 7, 3, 7, 6, 2, 1, 2, 6, 5, 0, 3, 2, 1,
];

const OCT_POINTS: [[f64; 3]; 6] = [
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [0.0, 0.0, -1.4142135623731],
    [0.0, 0.0, 1.4142135623731],
];
const OCT_VERTS: [i64; 24] = [
    4, 1, 0, 4, 2, 1, 4, 3, 2, 4, 0, 3, 0, 1, 5, 1, 2, 5, 2, 3, 5, 3, 0, 5,
];

const A_0: f64 = 0.61803398875;
const B: f64 = 0.381966011250;
const DODE_POINTS: [[f64; 3]; 20] = [
    [B, 0.0, 1.0],
    [-B, 0.0, 1.0],
    [B, 0.0, -1.0],
    [-B, 0.0, -1.0],
    [0.0, 1.0, -B],
    [0.0, 1.0, B],
    [0.0, -1.0, -B],
    [0.0, -1.0, B],
    [1.0, B, 0.0],
    [1.0, -B, 0.0],
    [-1.0, B, 0.0],
    [-1.0, -B, 0.0],
    [-A_0, A_0, A_0],
    [A_0, -A_0, A_0],
    [-A_0, -A_0, -A_0],
    [A_0, A_0, -A_0],
    [A_0, A_0, A_0],
    [-A_0, A_0, -A_0],
    [-A_0, -A_0, A_0],
    [A_0, -A_0, -A_0],
];
const DODE_VERTS: [i64; 60] = [
    0, 16, 5, 12, 1, 1, 18, 7, 13, 0, 2, 19, 6, 14, 3, 3, 17, 4, 15, 2, 4, 5, 16, 8, 15, 5, 4, 17,
    10, 12, 6, 7, 18, 11, 14, 7, 6, 19, 9, 13, 8, 16, 0, 13, 9, 9, 19, 2, 15, 8, 10, 17, 3, 14, 11,
    11, 18, 1, 12, 10,
];

const C: f64 = 0.5;
const D: f64 = 0.30901699;
const ICOSA_POINTS: [[f64; 3]; 12] = [
    [0.0, D, -C],
    [0.0, D, C],
    [0.0, -D, C],
    [-D, C, 0.0],
    [-D, -C, 0.0],
    [D, C, 0.0],
    [D, -C, 0.0],
    [0.0, -D, -C],
    [C, 0.0, D],
    [-C, 0.0, D],
    [-C, 0.0, -D],
    [C, 0.0, -D],
];
const ICOSA_VERTS: [i64; 60] = [
    0, 3, 5, 1, 5, 3, 1, 9, 2, 1, 2, 8, 0, 11, 7, 0, 7, 10, 2, 4, 6, 7, 6, 4, 3, 10, 9, 4, 9, 10,
    5, 8, 11, 6, 11, 8, 1, 3, 9, 1, 8, 5, 0, 10, 3, 0, 5, 11, 7, 4, 10, 7, 11, 6, 2, 9, 4, 2, 6, 8,
];

fn build_poly_data(
    solid_points: &[[f64; 3]],
    solid_verts: &[i64],
    cell_size: usize,
    solid_scale: f64,
) -> PolyData {
    let mut points = Points::<f64>::new();
    for point in solid_points {
        points.push([
            solid_scale * point[0],
            solid_scale * point[1],
            solid_scale * point[2],
        ]);
    }

    let mut polys = CellArray::new();
    let mut colors = Vec::with_capacity(solid_verts.len() / cell_size);
    for (i, cell) in solid_verts.chunks_exact(cell_size).enumerate() {
        polys.push_cell(cell);
        colors.push(i as i32);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.cell_data_mut()
        .add_array(AnyDataArray::I32(DataArray::from_vec(
            "FaceIndex",
            colors,
            1,
        )));
    pd.cell_data_mut().set_active_scalars("FaceIndex");
    pd
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
    fn cube() {
        let pd = platonic_solid(PlatonicSolidType::Cube);
        assert_eq!(pd.points.len(), 8);
        assert_eq!(pd.polys.num_cells(), 6);
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
