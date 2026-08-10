use crate::data::PolyData;

/// Extract closed boundary loops from a mesh, together with the loop count.
///
/// Boundary edges are those used by exactly one polygon. Thin wrapper over the
/// single loop-tracing implementation in
/// [`crate::filters::mesh::extract_boundary::extract_boundary_loops`]; the
/// second tuple element is simply the number of traced line cells.
pub fn extract_boundary_loops(input: &PolyData) -> (PolyData, usize) {
    let pd = crate::filters::mesh::extract_boundary::extract_boundary_loops(input);
    let num_loops = pd.lines.num_cells();
    (pd, num_loops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_one_loop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (result, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 1);
        assert_eq!(result.lines.num_cells(), 1);
        let line = result.lines.cell(0);
        assert_eq!(line.len(), 4);
        assert_eq!(line.first(), line.last());
        // The loop should contain 3 unique boundary vertices
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn closed_tetrahedron_no_loops() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 0);
    }

    #[test]
    fn two_separate_triangles_two_loops() {
        let mut pd = PolyData::new();
        // First triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        // Second triangle (separate)
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([4.0, 0.0, 0.0]);
        pd.points.push([3.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 2);
    }

    #[test]
    fn vertex_touching_triangles_are_two_loops() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);

        let (result, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 2);
        assert_eq!(result.lines.num_cells(), 2);
    }

    #[test]
    fn invalid_polygon_ids_are_ignored() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 100]);

        let (_, count) = extract_boundary_loops(&pd);
        assert_eq!(count, 1);
    }
}
