//! Extract and analyze the edge network of a mesh as a graph.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract unique edges as a line PolyData with edge length data.
pub fn extract_edge_network(mesh: &PolyData) -> PolyData {
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut lengths = Vec::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if seen.insert((a.min(b), a.max(b))) {
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
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt(),
                );
            }
        }
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
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if seen.insert((a.min(b), a.max(b))) {
                degree[a] += 1.0;
                degree[b] += 1.0;
            }
        }
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
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if !seen.insert((a.min(b), a.max(b))) {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let len = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
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
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
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
