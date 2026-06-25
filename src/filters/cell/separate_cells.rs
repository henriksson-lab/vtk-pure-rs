use crate::data::PolyData;

/// Separate all cells so they don't share any vertices.
///
/// Uses the `vtkShrinkPolyData` topology at factor 1.0: each output cell gets
/// its own copy of its vertices, polylines are split into line segments, and
/// triangle strips are split into triangle polygons.
pub fn separate_cells(input: &PolyData) -> PolyData {
    crate::filters::cell::shrink::shrink(input, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_vertices_duplicated() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // shared by both tris
        pd.points.push([1.0, 0.0, 0.0]); // shared
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = separate_cells(&pd);
        assert_eq!(result.points.len(), 6); // 3 + 3, no sharing
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn single_cell_unchanged_count() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = separate_cells(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = separate_cells(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn separates_all_polydata_cell_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[0, 1]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = separate_cells(&pd);
        assert_eq!(result.points.len(), 9);
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.strips.num_cells(), 0);
    }
}
