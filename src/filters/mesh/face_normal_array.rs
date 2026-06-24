use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-face (cell) normals via cross product and add as cell data.
///
/// For each polygon, uses the first three vertices to compute the face normal
/// via the cross product of two edge vectors. Adds a 3-component "FaceNormals"
/// array to cell data.
pub fn compute_face_normals(input: &PolyData) -> PolyData {
    let mut normals: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);

        // Edge vectors
        let u: [f64; 3] = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v: [f64; 3] = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        // Cross product
        let nx: f64 = u[1] * v[2] - u[2] * v[1];
        let ny: f64 = u[2] * v[0] - u[0] * v[2];
        let nz: f64 = u[0] * v[1] - u[1] * v[0];

        let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-20 {
            normals.extend_from_slice(&[nx / len, ny / len, nz / len]);
        } else {
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceNormals",
            normals,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xy_plane_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_face_normals(&pd);
        let arr = result.cell_data().get_array("FaceNormals").unwrap();
        assert_eq!(arr.num_tuples(), 1);
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        // Normal should point in +z direction
        assert!(val[2] > 0.99, "expected +z normal, got {:?}", val);
        assert!(val[0].abs() < 1e-10);
        assert!(val[1].abs() < 1e-10);
    }

    #[test]
    fn xz_plane_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_face_normals(&pd);
        let arr = result.cell_data().get_array("FaceNormals").unwrap();
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        // Normal should point in -y direction
        assert!(val[1] < -0.99, "expected -y normal, got {:?}", val);
    }

    #[test]
    fn multiple_faces() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = compute_face_normals(&pd);
        let arr = result.cell_data().get_array("FaceNormals").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        // First face normal is +z, second is -y
        let mut n0 = [0.0f64; 3];
        let mut n1 = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut n0);
        arr.tuple_as_f64(1, &mut n1);
        assert!(n0[2] > 0.99);
        assert!(n1[1] < -0.99);
    }
}
