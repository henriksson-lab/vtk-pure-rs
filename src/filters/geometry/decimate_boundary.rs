use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Identify and mark boundary points (and cells) on a mesh.
///
/// Re-exported from [`crate::filters::geometry::mark_boundary`], which holds the
/// single implementation (the faithful `vtkMarkBoundaryFilter` translation:
/// unsigned-char "BoundaryPoints" and "BoundaryCells" arrays).
pub use crate::filters::geometry::mark_boundary::mark_boundary;

/// Extract only boundary edges as line segments.
pub fn extract_boundary(input: &PolyData) -> PolyData {
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut point_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            let ma = *point_map.entry(a).or_insert_with(|| {
                let idx = out_points.len() as i64;
                out_points.push(input.points.get(a as usize));
                idx
            });
            let mb = *point_map.entry(b).or_insert_with(|| {
                let idx = out_points.len() as i64;
                out_points.push(input.points.get(b as usize));
                idx
            });
            out_lines.push_cell(&[ma, mb]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_pair_has_interior() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = mark_boundary(&pd);
        let arr = result.point_data().get_array("BoundaryPoints").unwrap();
        let mut buf = [0.0f64];
        // Edge 0-2 is shared -> points 0 and 2 are on boundary edges AND shared edge
        // All 4 outer edges are boundary, so all points are boundary
        // But edge 0-2 is shared (count=2), so it's not boundary
        // Boundary edges: 0-1, 1-2, 2-3, 3-0 -> all points are boundary
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

    #[test]
    fn extract_boundary_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = extract_boundary(&pd);
        assert_eq!(result.lines.num_cells(), 3); // 3 boundary edges
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = mark_boundary(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
