use crate::data::{CellArray, PolyData};
use std::collections::{BTreeMap, BTreeSet};

fn add_poly_edges<F>(input: &PolyData, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    let number_of_points = input.points.len();
    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            if valid_point_id(a, number_of_points) && valid_point_id(b, number_of_points) {
                add_edge(a, b);
            }
        }
    }
}

fn add_strip_edges<F>(input: &PolyData, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    let number_of_points = input.points.len();
    for cell in input.strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..(cell.len() - 2) {
            let a = cell[i];
            let b = cell[i + 1];
            let c = cell[i + 2];
            if valid_point_id(a, number_of_points)
                && valid_point_id(b, number_of_points)
                && valid_point_id(c, number_of_points)
            {
                add_edge(a, b);
                add_edge(b, c);
                add_edge(c, a);
            }
        }
    }
}

fn add_line_edges<F>(input: &PolyData, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    let number_of_points = input.points.len();
    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..(cell.len() - 1) {
            let a = cell[i];
            let b = cell[i + 1];
            if valid_point_id(a, number_of_points) && valid_point_id(b, number_of_points) {
                add_edge(a, b);
            }
        }
    }
}

fn valid_point_id(id: i64, number_of_points: usize) -> bool {
    id >= 0 && (id as usize) < number_of_points
}

fn sorted_edge(a: i64, b: i64) -> Option<(i64, i64)> {
    if a == b {
        None
    } else if a < b {
        Some((a, b))
    } else {
        Some((b, a))
    }
}

/// Convert a polygon mesh to its wireframe representation.
///
/// Extracts all unique edges as line cells. Points are shared.
pub fn wireframe(input: &PolyData) -> PolyData {
    let mut edges = BTreeSet::<(i64, i64)>::new();
    add_line_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            edges.insert(edge);
        }
    });
    add_poly_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            edges.insert(edge);
        }
    });
    add_strip_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            edges.insert(edge);
        }
    });

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
    let mut edge_count = BTreeMap::<(i64, i64), usize>::new();
    add_poly_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    });
    add_strip_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    });

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
    let mut edge_count = BTreeMap::<(i64, i64), usize>::new();
    add_poly_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    });
    add_strip_edges(input, |a, b| {
        if let Some(edge) = sorted_edge(a, b) {
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    });

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
    fn wireframe_includes_lines_and_preserves_all_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let wf = wireframe(&pd);
        assert_eq!(wf.lines.num_cells(), 2);
        assert_eq!(wf.points.len(), 4);
    }

    #[test]
    fn boundary_and_internal_ignore_input_lines() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        assert_eq!(boundary_wireframe(&pd).lines.num_cells(), 0);
        assert_eq!(internal_wireframe(&pd).lines.num_cells(), 0);
    }

    #[test]
    fn wireframe_includes_triangle_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let wf = wireframe(&pd);
        assert_eq!(wf.lines.num_cells(), 5);
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
