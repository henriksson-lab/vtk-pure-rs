//! Extract vertices whose scalar value is in a given range as a point cloud.
use crate::data::{CellArray, Points, PolyData};

pub fn extract_vertices_by_scalar(
    mesh: &PolyData,
    scalar_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= min_val && buf[0] <= max_val {
            let idx = pts.len();
            pts.push(mesh.points.get(i).try_into().unwrap());
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.verts = verts;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_extract_verts() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let r = extract_vertices_by_scalar(&mesh, "v", 3.0, 7.0);
        assert_eq!(r.points.len(), 1); // only vertex 1 (value 5.0)
    }
}
