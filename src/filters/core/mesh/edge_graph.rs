use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use std::collections::HashSet;

/// Extract the edge graph of a mesh as a PolyData with line cells.
///
/// Each unique edge becomes a line segment. Points are shared with input.
pub fn edge_graph(input: &PolyData) -> PolyData {
    let mut edges: HashSet<(i64, i64)> = HashSet::new();

    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut edges, input.points.len(), edge[0], edge[1]);
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(
                &mut edges,
                input.points.len(),
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }

    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edges, input.points.len(), tri[0], tri[1]);
            insert_edge(&mut edges, input.points.len(), tri[1], tri[2]);
            insert_edge(&mut edges, input.points.len(), tri[2], tri[0]);
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

/// Compute vertex degree (number of edges) for each vertex.
/// Adds "Degree" scalar.
pub fn vertex_degree(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut degree = vec![0.0f64; n];
    let mut counted: HashSet<(usize, usize)> = HashSet::new();

    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            insert_index_edge(&mut counted, &mut degree, n, edge[0], edge[1]);
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_index_edge(
                &mut counted,
                &mut degree,
                n,
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }

    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_index_edge(&mut counted, &mut degree, n, tri[0], tri[1]);
            insert_index_edge(&mut counted, &mut degree, n, tri[1], tri[2]);
            insert_index_edge(&mut counted, &mut degree, n, tri[2], tri[0]);
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Degree", degree, 1)));
    pd
}

/// Count total number of unique edges.
pub fn edge_count(input: &PolyData) -> usize {
    let mut edges: HashSet<(i64, i64)> = HashSet::new();
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut edges, input.points.len(), edge[0], edge[1]);
        }
    }
    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(
                &mut edges,
                input.points.len(),
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }
    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edges, input.points.len(), tri[0], tri[1]);
            insert_edge(&mut edges, input.points.len(), tri[1], tri[2]);
            insert_edge(&mut edges, input.points.len(), tri[2], tri[0]);
        }
    }
    edges.len()
}

fn insert_edge(edges: &mut HashSet<(i64, i64)>, n_points: usize, a: i64, b: i64) {
    if a == b || !valid_point_id(a, n_points) || !valid_point_id(b, n_points) {
        return;
    }
    edges.insert(if a < b { (a, b) } else { (b, a) });
}

fn insert_index_edge(
    edges: &mut HashSet<(usize, usize)>,
    degree: &mut [f64],
    n_points: usize,
    a: i64,
    b: i64,
) {
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
    if edges.insert(key) {
        degree[a] += 1.0;
        degree[b] += 1.0;
    }
}

fn valid_point_id(id: i64, n_points: usize) -> bool {
    usize::try_from(id).is_ok_and(|id| id < n_points)
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let graph = edge_graph(&pd);
        assert_eq!(graph.lines.num_cells(), 3);
        assert_eq!(edge_count(&pd), 3);
    }

    #[test]
    fn shared_edge() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(edge_count(&pd), 5); // 3+3-1 shared = 5
    }

    #[test]
    fn degree_regular() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_degree(&pd);
        let arr = result.point_data().get_array("Degree").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 2.0);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(edge_count(&pd), 0);
    }

    #[test]
    fn line_cells_contribute_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let graph = edge_graph(&pd);
        assert_eq!(graph.lines.num_cells(), 2);
        assert_eq!(edge_count(&pd), 2);

        let result = vertex_degree(&pd);
        let arr = result.point_data().get_array("Degree").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn triangle_strips_contribute_triangle_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        assert_eq!(edge_count(&pd), 5);
        assert_eq!(edge_graph(&pd).lines.num_cells(), 5);
    }
}
