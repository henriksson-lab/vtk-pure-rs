use crate::data::{CellArray, PolyData};
use std::collections::BTreeSet;

/// Convert a polygon mesh to its wireframe representation.
///
/// Extracts all unique edges as line cells. Points are shared.
pub fn wireframe(input: &PolyData) -> PolyData {
    let mut edges: BTreeSet<(i64, i64)> = BTreeSet::new();
    let number_of_points = input.points.len();
    for cell in input.lines.iter() {
        add_open_cell_edges(cell, number_of_points, |a, b| {
            edges.insert((a.min(b), a.max(b)));
        });
    }
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, number_of_points, |a, b| {
            edges.insert((a.min(b), a.max(b)));
        });
    }
    for cell in input.strips.iter() {
        add_triangle_strip_edges(cell, number_of_points, |a, b| {
            edges.insert((a.min(b), a.max(b)));
        });
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
    let number_of_points = input.points.len();
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, number_of_points, |a, b| {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        });
    }
    for cell in input.strips.iter() {
        add_triangle_strip_edges(cell, number_of_points, |a, b| {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        });
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
    let number_of_points = input.points.len();
    for cell in input.polys.iter() {
        add_closed_cell_edges(cell, number_of_points, |a, b| {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        });
    }
    for cell in input.strips.iter() {
        add_triangle_strip_edges(cell, number_of_points, |a, b| {
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        });
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

fn valid_point_id(id: i64, number_of_points: usize) -> bool {
    id >= 0 && (id as usize) < number_of_points
}

fn add_closed_cell_edges<F>(cell: &[i64], number_of_points: usize, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_valid_edge(
            cell[i],
            cell[(i + 1) % cell.len()],
            number_of_points,
            &mut add_edge,
        );
    }
}

fn add_open_cell_edges<F>(cell: &[i64], number_of_points: usize, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    for edge in cell.windows(2) {
        add_valid_edge(edge[0], edge[1], number_of_points, &mut add_edge);
    }
}

fn add_triangle_strip_edges<F>(cell: &[i64], number_of_points: usize, mut add_edge: F)
where
    F: FnMut(i64, i64),
{
    if cell.len() < 3 {
        return;
    }
    for i in 0..cell.len() - 2 {
        let tri = if i % 2 == 0 {
            [cell[i], cell[i + 1], cell[i + 2]]
        } else {
            [cell[i + 1], cell[i], cell[i + 2]]
        };
        add_valid_edge(tri[0], tri[1], number_of_points, &mut add_edge);
        add_valid_edge(tri[1], tri[2], number_of_points, &mut add_edge);
        add_valid_edge(tri[2], tri[0], number_of_points, &mut add_edge);
    }
}

fn add_valid_edge<F>(a: i64, b: i64, number_of_points: usize, add_edge: &mut F)
where
    F: FnMut(i64, i64),
{
    if valid_point_id(a, number_of_points) && valid_point_id(b, number_of_points) && a != b {
        add_edge(a, b);
    }
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
