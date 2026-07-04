//! Extract and analyze the edge network of a mesh as a graph.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::{BTreeSet, HashMap};

/// Extract unique edges as a line PolyData with edge length data.
pub fn extract_edge_network(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut lengths = Vec::new();
    let mut pt_map: HashMap<usize, usize> = HashMap::new();

    for (a, b) in unique_edges(mesh) {
        let ia = *pt_map.entry(a).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(a));
            i
        });
        let ib = *pt_map.entry(b).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(b));
            i
        });
        lines.push_cell(&[ia as i64, ib as i64]);
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        lengths.push(
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt(),
        );
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeLength",
            lengths,
            1,
        )));
    result
}

/// Compute vertex degree (number of edges per vertex).
pub fn vertex_degree(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut degree = vec![0.0f64; n];
    for (a, b) in unique_edges(mesh) {
        degree[a] += 1.0;
        degree[b] += 1.0;
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Degree", degree, 1)));
    result
}

/// Extract edges longer than a threshold.
pub fn extract_long_edges(mesh: &PolyData, min_length: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map: HashMap<usize, usize> = HashMap::new();

    for (a, b) in unique_edges(mesh) {
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let len =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        if len >= min_length {
            let ia = *pt_map.entry(a).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(a));
                i
            });
            let ib = *pt_map.entry(b).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(b));
                i
            });
            lines.push_cell(&[ia as i64, ib as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

fn unique_edges(mesh: &PolyData) -> BTreeSet<(usize, usize)> {
    let mut edges = BTreeSet::new();
    let n_points = mesh.points.len();

    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut edges, n_points, edge[0], edge[1]);
        }
    }

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(&mut edges, n_points, cell[i], cell[(i + 1) % cell.len()]);
        }
    }

    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edges, n_points, tri[0], tri[1]);
            insert_edge(&mut edges, n_points, tri[1], tri[2]);
            insert_edge(&mut edges, n_points, tri[2], tri[0]);
        }
    }

    edges
}

fn insert_edge(edges: &mut BTreeSet<(usize, usize)>, n_points: usize, a: i64, b: i64) {
    let (Some(a), Some(b)) = (
        valid_point_index(a, n_points),
        valid_point_index(b, n_points),
    ) else {
        return;
    };
    if a != b {
        edges.insert((a.min(b), a.max(b)));
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edge_net() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let net = extract_edge_network(&mesh);
        assert_eq!(net.lines.num_cells(), 3);
        assert!(net.cell_data().get_array("EdgeLength").is_some());
    }
    #[test]
    fn degree() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = vertex_degree(&mesh);
        let arr = result.point_data().get_array("Degree").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0); // vertex 1 touches 3 edges
    }
    #[test]
    fn long_edges() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extract_long_edges(&mesh, 5.0);
        assert!(result.lines.num_cells() >= 1); // the 10-unit edge
    }
}
