//! Count the number of faces incident to each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_count_per_vertex(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut counts = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        for &v in &cell[..] {
            let vi = v as usize;
            if vi < n {
                counts[vi] += 1.0;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceCount",
            counts,
            1,
        )));
    result.point_data_mut().set_active_scalars("FaceCount");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_face_count() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = face_count_per_vertex(&mesh);
        let arr = r.point_data().get_array("FaceCount").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b);
        assert_eq!(b[0], 2.0); // vertex 1 is in both faces
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 1.0); // vertex 0 is in one face
    }
}
