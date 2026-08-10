use crate::data::PolyData;

pub use crate::filters::mesh::extract_boundary::extract_boundary_loops;

/// Count the number of boundary loops.
pub fn num_boundary_loops(input: &PolyData) -> usize {
    let loops = extract_boundary_loops(input);
    loops.lines.num_cells()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_surface_no_loops() {
        // Tetrahedron: 4 triangles, all edges shared
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(num_boundary_loops(&pd), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(num_boundary_loops(&pd), 0);
    }
}
