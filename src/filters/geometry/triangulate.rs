use crate::data::{CellArray, PolyData};

/// Convert all polygons to triangles using fan triangulation.
///
/// Triangles pass through unchanged. Quads and larger polygons are decomposed
/// into triangle fans from the first vertex. Triangle strips are also decomposed.
pub fn triangulate(input: &PolyData) -> PolyData {
    // Fast path: if already all triangles and no strips, just clone
    let has_strips = !input.strips.is_empty();
    let all_tri = input.polys.is_empty() || all_cells_are_size(&input.polys, 3);
    if all_tri && !has_strips {
        return input.clone();
    }

    let mut output = input.clone();

    // Triangulate polygons
    if !input.polys.is_empty() {
        output.polys = triangulate_cells(&input.polys);
    }

    // Decompose triangle strips
    if has_strips {
        let tri_from_strips = decompose_strips(&input.strips);
        // Append strip-derived triangles to polys
        for cell in tri_from_strips.iter() {
            output.polys.push_cell(cell);
        }
        output.strips = CellArray::new();
    }

    output
}

fn triangulate_cells(polys: &CellArray) -> CellArray {
    // Fast path: if all cells are already triangles, return a clone directly.
    // This matches VTK's TriangleFilter behavior which is a no-op on triangle meshes.
    if all_cells_are_size(polys, 3) {
        return polys.clone();
    }

    let mut out_cells = 0usize;
    let mut out_conn_len = 0usize;
    for cell in polys.iter() {
        if cell.len() >= 3 {
            let n = cell.len() - 2;
            out_cells += n;
            out_conn_len += n * 3;
        }
    }

    if out_cells == 0 {
        return CellArray::new();
    }

    let mut offsets = Vec::with_capacity(out_cells + 1);
    let mut conn = Vec::with_capacity(out_conn_len);
    offsets.push(0i64);

    for cell in polys.iter() {
        if cell.len() < 3 {
            // Degenerate, skip
            continue;
        }
        if cell.len() == 3 {
            // Already a triangle
            conn.extend_from_slice(cell);
            offsets.push(conn.len() as i64);
        } else {
            // Fan triangulation from vertex 0
            for i in 1..cell.len() - 1 {
                conn.extend_from_slice(&[cell[0], cell[i], cell[i + 1]]);
                offsets.push(conn.len() as i64);
            }
        }
    }

    CellArray::from_raw(offsets, conn)
}

fn decompose_strips(strips: &CellArray) -> CellArray {
    let out_cells: usize = strips
        .iter()
        .map(|strip| strip.len().saturating_sub(2))
        .sum();
    if out_cells == 0 {
        return CellArray::new();
    }

    let mut offsets = Vec::with_capacity(out_cells + 1);
    let mut conn = Vec::with_capacity(out_cells * 3);
    offsets.push(0i64);

    for strip in strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            if i % 2 == 0 {
                conn.extend_from_slice(&[strip[i], strip[i + 1], strip[i + 2]]);
            } else {
                // Flip winding for odd triangles to maintain consistent orientation
                conn.extend_from_slice(&[strip[i + 1], strip[i], strip[i + 2]]);
            }
            offsets.push(conn.len() as i64);
        }
    }

    CellArray::from_raw(offsets, conn)
}

fn all_cells_are_size(cells: &CellArray, size: i64) -> bool {
    cells
        .offsets()
        .windows(2)
        .all(|pair| pair[1] - pair[0] == size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_passthrough() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = triangulate(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn quad_to_triangles() {
        let mut pd = PolyData::new();
        pd.points = crate::data::Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = triangulate(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[0, 2, 3]);
    }

    #[test]
    fn strip_decomposition() {
        let mut pd = PolyData::new();
        pd.points = crate::data::Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = triangulate(&pd);
        assert!(result.strips.is_empty());
        assert_eq!(result.polys.num_cells(), 2);
    }
}
