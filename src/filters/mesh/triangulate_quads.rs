use crate::data::{CellArray, PolyData};

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx: f64 = a[0] - b[0];
    let dy: f64 = a[1] - b[1];
    let dz: f64 = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Triangulate quad faces in a PolyData by splitting along the shorter diagonal.
///
/// Triangles and other non-quad cells are passed through unchanged.
/// Each quad (4-vertex polygon) is split into two triangles along the
/// diagonal that is shorter, producing better-shaped triangles.
pub fn triangulate_quads(input: &PolyData) -> PolyData {
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() == 4 {
            let p0 = input.points.get(cell[0] as usize);
            let p1 = input.points.get(cell[1] as usize);
            let p2 = input.points.get(cell[2] as usize);
            let p3 = input.points.get(cell[3] as usize);

            let diag_02: f64 = dist_sq(p0, p2);
            let diag_13: f64 = dist_sq(p1, p3);

            if diag_02 <= diag_13 {
                // Split along 0-2
                out_polys.push_cell(&[cell[0], cell[1], cell[2]]);
                out_polys.push_cell(&[cell[0], cell[2], cell[3]]);
            } else {
                // Split along 1-3
                out_polys.push_cell(&[cell[0], cell[1], cell[3]]);
                out_polys.push_cell(&[cell[1], cell[2], cell[3]]);
            }
        } else {
            out_polys.push_cell(cell);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.polys = out_polys;
    // Copy verts, lines, strips unchanged
    pd.verts = input.verts.clone();
    pd.lines = input.lines.clone();
    pd.strips = input.strips.clone();
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    fn make_quad() -> PolyData {
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([1.0, 1.0, 0.0]);
        pts.push([0.0, 1.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2, 3]);

        let mut pd = PolyData::new();
        pd.points = pts;
        pd.polys = polys;
        pd
    }

    #[test]
    fn quad_becomes_two_triangles() {
        let input = make_quad();
        let result = triangulate_quads(&input);
        assert_eq!(result.polys.num_cells(), 2);
        // Points should be the same (shared)
        assert_eq!(result.points.len(), 4);
        // Each cell should have 3 vertices
        for cell in result.polys.iter() {
            assert_eq!(cell.len(), 3);
        }
    }

    #[test]
    fn triangles_pass_through() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = triangulate_quads(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 3);
    }

    #[test]
    fn shorter_diagonal_split() {
        // Make a non-square quad where diagonal 0-2 is shorter than 1-3
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([2.0, 0.0, 0.0]);
        pts.push([0.5, 0.5, 0.0]); // close to vertex 0
        pts.push([0.0, 2.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2, 3]);

        let mut pd = PolyData::new();
        pd.points = pts;
        pd.polys = polys;

        let result = triangulate_quads(&pd);
        assert_eq!(result.polys.num_cells(), 2);

        // Diagonal 0-2 (distance sqrt(0.5)) is shorter than 1-3 (distance sqrt(8))
        // so should split along 0-2: triangles (0,1,2) and (0,2,3)
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert!(cells[0].contains(&0) && cells[0].contains(&1) && cells[0].contains(&2));
        assert!(cells[1].contains(&0) && cells[1].contains(&2) && cells[1].contains(&3));
    }
}
