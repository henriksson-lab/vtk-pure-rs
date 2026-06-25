use crate::data::{CellArray, PolyData};
use std::collections::{HashMap, HashSet};

/// Close all boundary loops in a mesh by capping them with fan triangulation.
///
/// Mirrors the main `vtkFillHolesFilter` flow for polygonal cells: boundary
/// edges are linked into simple loops and each valid loop is triangulated using
/// existing point ids. No new cap-center points are created.
pub fn close_holes(input: &PolyData) -> PolyData {
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        count_cell_edges(cell, &mut edge_count);
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = [strip[i], strip[i + 1], strip[i + 2]];
            count_cell_edges(&tri, &mut edge_count);
        }
    }

    let boundary_edges: Vec<(i64, i64)> = edge_count
        .into_iter()
        .filter_map(|(edge, count)| (count == 1).then_some(edge))
        .collect();

    let mut edge_links: HashMap<i64, Vec<usize>> = HashMap::new();
    for (edge_id, &(a, b)) in boundary_edges.iter().enumerate() {
        edge_links.entry(a).or_default().push(edge_id);
        edge_links.entry(b).or_default().push(edge_id);
    }

    let mut out_polys = input.polys.clone();
    let mut visited = vec![false; boundary_edges.len()];

    for cell_id in 0..boundary_edges.len() {
        if visited[cell_id] {
            continue;
        }

        visited[cell_id] = true;
        let (start_id, mut end_id) = boundary_edges[cell_id];
        let mut current_cell_id = cell_id;
        let mut polygon = vec![start_id];
        let mut valid = true;

        while start_id != end_id && valid {
            polygon.push(end_id);
            let neighbors = match edge_links.get(&end_id) {
                Some(edges) => edges,
                None => {
                    valid = false;
                    break;
                }
            };

            let next_edges: Vec<usize> = neighbors
                .iter()
                .copied()
                .filter(|&edge_id| edge_id != current_cell_id && !visited[edge_id])
                .collect();

            if next_edges.len() != 1 {
                valid = false;
            } else {
                let nei_id = next_edges[0];
                visited[nei_id] = true;
                let (p0, p1) = boundary_edges[nei_id];
                end_id = if p0 != end_id { p0 } else { p1 };
                current_cell_id = nei_id;
            }
        }

        if valid && polygon.len() >= 3 && is_simple_loop(&polygon) {
            triangulate_loop(&polygon, &mut out_polys);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.verts = input.verts.clone();
    pd.lines = input.lines.clone();
    pd.polys = out_polys;
    pd.strips = input.strips.clone();
    *pd.point_data_mut() = input.point_data().clone();
    pd
}

fn count_cell_edges(cell: &[i64], edge_count: &mut HashMap<(i64, i64), usize>) {
    for i in 0..cell.len() {
        let a = cell[i];
        let b = cell[(i + 1) % cell.len()];
        let key = if a < b { (a, b) } else { (b, a) };
        *edge_count.entry(key).or_insert(0) += 1;
    }
}

fn is_simple_loop(polygon: &[i64]) -> bool {
    let mut seen = HashSet::with_capacity(polygon.len());
    polygon.iter().all(|&point_id| seen.insert(point_id))
}

fn triangulate_loop(polygon: &[i64], polys: &mut CellArray) {
    if polygon.len() == 3 {
        polys.push_cell(polygon);
    } else {
        for i in 1..polygon.len() - 1 {
            if polygon[0] != polygon[i]
                && polygon[i] != polygon[i + 1]
                && polygon[0] != polygon[i + 1]
            {
                polys.push_cell(&[polygon[0], polygon[i], polygon[i + 1]]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_single_hole() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = close_holes(&pd);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 3);
    }

    #[test]
    fn already_closed() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = close_holes(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = close_holes(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
