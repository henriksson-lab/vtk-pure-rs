use crate::data::{CellArray, Points, PolyData};
use std::collections::HashSet;

/// Convert a polygon mesh to its wireframe representation.
///
/// Extracts all unique edges as line cells. Points are shared.
pub fn wireframe(input: &PolyData) -> PolyData {
    let mut edges: HashSet<(i64, i64)> = HashSet::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            edges.insert(if a < b { (a, b) } else { (b, a) });
        }
    }

    let mut out_lines = CellArray::new();
    for &(a, b) in &edges {
        out_lines.push_cell(&[a, b]);
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = out_lines;
    pd
}

/// Extract only boundary edges as a wireframe.
pub fn boundary_wireframe(input: &PolyData) -> PolyData {
    let mut edge_count: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut out_lines = CellArray::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            out_lines.push_cell(&[a, b]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = out_lines;
    pd
}

/// Extract only internal (non-boundary) edges.
pub fn internal_wireframe(input: &PolyData) -> PolyData {
    let mut edge_count: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut out_lines = CellArray::new();
    for (&(a, b), &c) in &edge_count {
        if c >= 2 {
            out_lines.push_cell(&[a, b]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wireframe_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let wf = wireframe(&pd);
        assert_eq!(wf.lines.num_cells(), 3);
    }

    #[test]
    fn boundary_vs_internal() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let bw = boundary_wireframe(&pd);
        let iw = internal_wireframe(&pd);
        assert_eq!(iw.lines.num_cells(), 1); // shared edge 0-2
        assert_eq!(bw.lines.num_cells(), 4); // 4 boundary edges
    }

    #[test]
    fn closed_mesh_no_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(boundary_wireframe(&pd).lines.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(wireframe(&pd).lines.num_cells(), 0);
    }
}
