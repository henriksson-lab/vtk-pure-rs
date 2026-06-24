//! Attach face index to cell data and propagate to point data.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_face_id(mesh: &PolyData) -> PolyData {
    let nc = mesh.polys.num_cells();
    let data: Vec<f64> = (0..nc).map(|i| i as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("FaceID", data, 1)));
    r
}
pub fn attach_face_vertex_count(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh.polys.iter().map(|c| c.len() as f64).collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexCount",
            data,
            1,
        )));
    r
}
pub fn attach_face_perimeter(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            let nc = cell.len();
            (0..nc)
                .map(|i| {
                    let a = mesh.points.get(cell[i] as usize);
                    let b = mesh.points.get(cell[(i + 1) % nc] as usize);
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
                })
                .sum()
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Perimeter", data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_id() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_face_id(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("FaceID")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
    #[test]
    fn test_vc() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_face_vertex_count(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("VertexCount")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
    }
    #[test]
    fn test_perim() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_face_perimeter(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Perimeter")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - (2.0 + 2.0f64.sqrt())).abs() < 1e-10);
    }
}
