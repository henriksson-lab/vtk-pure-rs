use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashSet;

/// Extract the edge graph of a mesh as a PolyData with line cells.
///
/// Each unique edge becomes a line segment. Points are shared with input.
pub fn edge_graph(input: &PolyData) -> PolyData {
    let mut edges: HashSet<(i64, i64)> = HashSet::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
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

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            if counted.insert(key) {
                degree[a] += 1.0;
                degree[b] += 1.0;
            }
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
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            edges.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    edges.len()
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
}
