//! Attach number of edges per vertex and per face.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn vertex_edge_count(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut counts = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                counts[a].insert(b);
                counts[b].insert(a);
            }
        }
    }
    let data: Vec<f64> = (0..n).map(|i| counts[i].len() as f64).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("EdgeCount", data, 1)));
    r
}
pub fn face_edge_count(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh.polys.iter().map(|c| c.len() as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("EdgeCount", data, 1)));
    r
}
pub fn total_unique_edges(mesh: &PolyData) -> usize {
    let mut edges = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
    }
    edges.len()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vert() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = vertex_edge_count(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("EdgeCount")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0);
    } // vertex 1 has 3 neighbors
    #[test]
    fn test_face() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_edge_count(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("EdgeCount")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
    }
    #[test]
    fn test_total() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert_eq!(total_unique_edges(&m), 5);
    }
}
