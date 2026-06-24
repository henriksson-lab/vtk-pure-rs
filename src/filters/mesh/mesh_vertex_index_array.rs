//! Attach vertex index as point data.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_vertex_index(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = (0..mesh.points.len()).map(|i| i as f64).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexIndex",
            data,
            1,
        )));
    r
}
pub fn attach_cell_index(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = (0..mesh.polys.num_cells()).map(|i| i as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CellIndex", data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vi() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_vertex_index(&m);
        assert!(r.point_data().get_array("VertexIndex").is_some());
    }
    #[test]
    fn test_ci() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_cell_index(&m);
        assert!(r.cell_data().get_array("CellIndex").is_some());
    }
}
