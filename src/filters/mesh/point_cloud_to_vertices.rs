use crate::data::{CellArray, Points, PolyData};

/// Convert a slice of 3D points into a PolyData with one vertex cell per point.
pub fn points_to_vertices(points: &[[f64; 3]]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();

    for (i, p) in points.iter().enumerate() {
        pts.push(*p);
        verts.push_cell(&[i as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = pts;
    pd.verts = verts;
    pd
}

/// Extract vertex positions from a PolyData's vertex cells.
///
/// Returns the point coordinates referenced by vertex cells. If the
/// PolyData has no vertex cells, returns an empty Vec.
pub fn vertices_to_points(poly: &PolyData) -> Vec<[f64; 3]> {
    let mut result: Vec<[f64; 3]> = Vec::new();
    for cell in poly.verts.iter() {
        for &id in cell {
            result.push(poly.points.get(id as usize));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_points() {
        let input: Vec<[f64; 3]> = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let pd = points_to_vertices(&input);
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.verts.num_cells(), 3);

        let output = vertices_to_points(&pd);
        assert_eq!(output.len(), 3);
        for i in 0..3 {
            for j in 0..3 {
                assert!((output[i][j] - input[i][j]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn empty_input() {
        let pd = points_to_vertices(&[]);
        assert_eq!(pd.points.len(), 0);
        assert_eq!(pd.verts.num_cells(), 0);

        let pts = vertices_to_points(&pd);
        assert!(pts.is_empty());
    }

    #[test]
    fn vertices_to_points_ignores_polys() {
        // A PolyData with only polygon cells should yield no vertex points
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let pts = vertices_to_points(&pd);
        assert!(pts.is_empty());
    }
}
