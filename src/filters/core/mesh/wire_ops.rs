//! Wireframe and edge extraction operations.

use crate::data::{CellArray, Points, PolyData};
use std::collections::{BTreeMap, BTreeSet, HashMap};

fn add_poly_edges<F>(mesh: &PolyData, mut add_edge: F)
where
    F: FnMut(usize, usize),
{
    let number_of_points = mesh.points.len();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            if let (Some(a), Some(b)) = (
                point_id(cell[i], number_of_points),
                point_id(cell[(i + 1) % nc], number_of_points),
            ) {
                add_edge(a, b);
            }
        }
    }
}

fn add_line_edges<F>(mesh: &PolyData, mut add_edge: F)
where
    F: FnMut(usize, usize),
{
    let number_of_points = mesh.points.len();
    for cell in mesh.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..(cell.len() - 1) {
            if let (Some(a), Some(b)) = (
                point_id(cell[i], number_of_points),
                point_id(cell[i + 1], number_of_points),
            ) {
                add_edge(a, b);
            }
        }
    }
}

fn add_strip_edges<F>(mesh: &PolyData, mut add_edge: F)
where
    F: FnMut(usize, usize),
{
    let number_of_points = mesh.points.len();
    for cell in mesh.strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..(cell.len() - 2) {
            if let (Some(a), Some(b), Some(c)) = (
                point_id(cell[i], number_of_points),
                point_id(cell[i + 1], number_of_points),
                point_id(cell[i + 2], number_of_points),
            ) {
                add_edge(a, b);
                add_edge(b, c);
                add_edge(c, a);
            }
        }
    }
}

fn point_id(id: i64, number_of_points: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < number_of_points {
        Some(id as usize)
    } else {
        None
    }
}

fn insert_edge_count(edge_counts: &mut BTreeMap<(usize, usize), usize>, a: usize, b: usize) {
    if a == b {
        return;
    }
    *edge_counts.entry((a.min(b), a.max(b))).or_insert(0) += 1;
}

fn copy_edge_points(
    mesh: &PolyData,
    point_map: &mut HashMap<usize, usize>,
    points: &mut Points<f64>,
    a: usize,
    b: usize,
) -> [i64; 2] {
    let ia = *point_map.entry(a).or_insert_with(|| {
        let i = points.len();
        points.push(mesh.points.get(a));
        i
    });
    let ib = *point_map.entry(b).or_insert_with(|| {
        let i = points.len();
        points.push(mesh.points.get(b));
        i
    });
    [ia as i64, ib as i64]
}

/// Extract wireframe (all edges as lines).
pub fn extract_wireframe(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut seen = BTreeSet::<(usize, usize)>::new();
    let mut pm = HashMap::<usize, usize>::new();

    add_line_edges(mesh, |a, b| {
        if a != b {
            seen.insert((a.min(b), a.max(b)));
        }
    });
    add_poly_edges(mesh, |a, b| {
        if a != b {
            seen.insert((a.min(b), a.max(b)));
        }
    });
    add_strip_edges(mesh, |a, b| {
        if a != b {
            seen.insert((a.min(b), a.max(b)));
        }
    });

    for (a, b) in seen {
        lines.push_cell(&copy_edge_points(mesh, &mut pm, &mut pts, a, b));
    }

    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

/// Extract only boundary edges as lines.
pub fn extract_boundary_wireframe(mesh: &PolyData) -> PolyData {
    let mut ec = BTreeMap::<(usize, usize), usize>::new();
    add_poly_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));
    add_strip_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm = HashMap::<usize, usize>::new();
    for (&(a, b), &count) in &ec {
        if count == 1 {
            lines.push_cell(&copy_edge_points(mesh, &mut pm, &mut pts, a, b));
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

/// Extract internal edges only (shared by 2 faces).
pub fn extract_internal_edges(mesh: &PolyData) -> PolyData {
    let mut ec = BTreeMap::<(usize, usize), usize>::new();
    add_poly_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));
    add_strip_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm = HashMap::<usize, usize>::new();
    for (&(a, b), &count) in &ec {
        if count == 2 {
            lines.push_cell(&copy_edge_points(mesh, &mut pm, &mut pts, a, b));
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

/// Count edges by type.
pub fn edge_counts(mesh: &PolyData) -> (usize, usize, usize) {
    let mut ec = BTreeMap::<(usize, usize), usize>::new();
    add_poly_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));
    add_strip_edges(mesh, |a, b| insert_edge_count(&mut ec, a, b));

    let boundary = ec.values().filter(|&&c| c == 1).count();
    let internal = ec.values().filter(|&&c| c == 2).count();
    let non_manifold = ec.values().filter(|&&c| c > 2).count();
    (boundary, internal, non_manifold)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wireframe() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let wf = extract_wireframe(&mesh);
        assert_eq!(wf.lines.num_cells(), 3);
    }
    #[test]
    fn wireframe_includes_input_lines() {
        let mut mesh =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        mesh.lines.push_cell(&[0, 1, 2]);
        let wf = extract_wireframe(&mesh);
        assert_eq!(wf.lines.num_cells(), 2);
        assert_eq!(wf.points.len(), 3);
    }
    #[test]
    fn wireframe_decomposes_triangle_strips() {
        let mut mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let wf = extract_wireframe(&mesh);
        assert_eq!(wf.lines.num_cells(), 5);
    }
    #[test]
    fn boundary() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let bnd = extract_boundary_wireframe(&mesh);
        assert_eq!(bnd.lines.num_cells(), 4); // 4 boundary edges
    }
    #[test]
    fn internal() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let int = extract_internal_edges(&mesh);
        assert_eq!(int.lines.num_cells(), 1); // edge 1-2 is shared
    }
    #[test]
    fn counts() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let (b, i, nm) = edge_counts(&mesh);
        assert_eq!(b, 4);
        assert_eq!(i, 1);
        assert_eq!(nm, 0);
    }
}
