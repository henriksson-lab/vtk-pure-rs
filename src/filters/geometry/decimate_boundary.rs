use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Identify and mark boundary points on a mesh.
///
/// A boundary edge is an edge that belongs to only one triangle.
/// Points on boundary edges are marked with 1.0 in a "BoundaryPoints"
/// point data array, others with 0.0.
pub fn mark_boundary(input: &PolyData) -> PolyData {
    let n = input.points.len();

    // Count how many triangles share each edge
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Points on boundary edges (count == 1)
    let mut boundary = vec![0.0f64; n];
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary[a as usize] = 1.0;
            boundary[b as usize] = 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryPoints",
            boundary,
            1,
        )));
    pd
}

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
    fn open_mesh_has_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = mark_boundary(&pd);
        let arr = result.point_data().get_array("BoundaryPoints").unwrap();
        let mut buf = [0.0f64];
        // Single triangle: all 3 edges are boundary
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

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
