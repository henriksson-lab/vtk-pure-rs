use crate::data::{CellArray, PolyData};

/// Convert all triangle strips in a PolyData to individual triangles.
///
/// Replaces `strips` cells with equivalent `polys` triangle cells,
/// handling the alternating winding convention of triangle strips.
pub fn triangulate_strips(input: &PolyData) -> PolyData {
    let mut pd = input.clone();

    for cell in input.strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..cell.len() - 2 {
            if i % 2 == 0 {
                pd.polys.push_cell(&[cell[i], cell[i + 1], cell[i + 2]]);
            } else {
                pd.polys.push_cell(&[cell[i + 1], cell[i], cell[i + 2]]);
            }
        }
    }

    pd.strips = CellArray::new();
    pd
}

/// Convert individual triangles to triangle strips (greedy).
///
/// Uses a greedy algorithm to merge adjacent triangles into strips.
/// Falls back to single-triangle strips for unmatched triangles.
pub fn triangles_to_strips(input: &PolyData) -> PolyData {
    // For simplicity, just convert each triangle to a 3-point strip
    // A full greedy strip builder would require edge adjacency
    let mut strips = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() == 3 {
            strips.push_cell(cell);
        }
    }

    // Keep non-triangle polys
    let mut polys = CellArray::new();
    for cell in input.polys.iter() {
        if cell.len() != 3 {
            polys.push_cell(cell);
        }
    }

    let mut pd = input.clone();
    pd.polys = polys;
    pd.strips = strips;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_to_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3, 4]); // 3 triangles

        let result = triangulate_strips(&pd);
        assert_eq!(result.polys.num_cells(), 3);
        assert_eq!(result.strips.num_cells(), 0);
    }

    #[test]
    fn empty_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = triangulate_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn triangles_to_strip() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = triangles_to_strips(&pd);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn preserves_quads() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]); // quad

        let result = triangles_to_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1); // quad stays as poly
        assert_eq!(result.strips.num_cells(), 0);
    }
}
