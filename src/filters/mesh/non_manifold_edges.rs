use std::collections::{BTreeMap, HashMap};

use crate::data::{CellArray, Points, PolyData};

/// Find non-manifold edges in a PolyData mesh.
///
/// Non-manifold edges are shared by more than 2 faces. Returns a new PolyData
/// containing line cells marking the non-manifold edges.
pub fn non_manifold_edges(input: &PolyData) -> PolyData {
    let edge_faces = build_edge_face_count(input);

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut point_map: HashMap<i64, i64> = HashMap::new();

    for (&(a, b), &count) in &edge_faces {
        if count > 2 {
            let new_a = map_point(a, input, &mut out_points, &mut point_map);
            let new_b = map_point(b, input, &mut out_points, &mut point_map);
            out_lines.push_cell(&[new_a, new_b]);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.lines = out_lines;
    result
}

/// Count the number of non-manifold edges in a PolyData mesh.
pub fn count_non_manifold_edges(input: &PolyData) -> usize {
    let edge_faces = build_edge_face_count(input);
    edge_faces.values().filter(|&&c| c > 2).count()
}

fn build_edge_face_count(input: &PolyData) -> BTreeMap<(i64, i64), usize> {
    let mut edge_faces: BTreeMap<(i64, i64), usize> = BTreeMap::new();
    let n_points = input.points.len();
    for cell in input.polys.iter() {
        insert_polygon_edges(&mut edge_faces, cell, n_points);
    }
    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edge_faces, tri[0], tri[1], n_points);
            insert_edge(&mut edge_faces, tri[1], tri[2], n_points);
            insert_edge(&mut edge_faces, tri[2], tri[0], n_points);
        }
    }
    edge_faces
}

fn insert_polygon_edges(
    edge_faces: &mut BTreeMap<(i64, i64), usize>,
    cell: &[i64],
    n_points: usize,
) {
    for i in 0..cell.len() {
        insert_edge(edge_faces, cell[i], cell[(i + 1) % cell.len()], n_points);
    }
}

fn insert_edge(edge_faces: &mut BTreeMap<(i64, i64), usize>, a: i64, b: i64, n_points: usize) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a == b {
        return;
    }
    let key = if a < b { (a, b) } else { (b, a) };
    *edge_faces.entry(key).or_insert(0) += 1;
}

fn valid_point_index(id: i64, n_points: usize) -> Option<i64> {
    usize::try_from(id)
        .ok()
        .filter(|&id| id < n_points)
        .map(|id| id as i64)
}

fn map_point(
    id: i64,
    input: &PolyData,
    out_points: &mut Points<f64>,
    point_map: &mut HashMap<i64, i64>,
) -> i64 {
    *point_map.entry(id).or_insert_with(|| {
        let idx = out_points.len() as i64;
        out_points.push(input.points.get(id as usize));
        idx
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifold_mesh() -> PolyData {
        // Two triangles sharing one edge — all edges manifold or boundary
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        )
    }

    fn make_non_manifold_mesh() -> PolyData {
        // Three triangles sharing the same edge (0-1) — that edge is non-manifold
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [0, 1, 4]],
        )
    }

    #[test]
    fn manifold_mesh_no_non_manifold_edges() {
        let pd = make_manifold_mesh();
        assert_eq!(count_non_manifold_edges(&pd), 0);
        let result = non_manifold_edges(&pd);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn detects_non_manifold_edge() {
        let pd = make_non_manifold_mesh();
        assert_eq!(count_non_manifold_edges(&pd), 1);
        let result = non_manifold_edges(&pd);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert_eq!(count_non_manifold_edges(&pd), 0);
        let result = non_manifold_edges(&pd);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn detects_non_manifold_edge_from_triangle_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.strips.push_cell(&[0, 1, 4]);

        assert_eq!(count_non_manifold_edges(&pd), 1);
        let result = non_manifold_edges(&pd);
        assert_eq!(result.lines.num_cells(), 1);
    }
}
