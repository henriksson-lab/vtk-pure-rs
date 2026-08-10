use crate::data::PolyData;

/// Compute per-face (cell) normals via cross product and add as cell data.
///
/// For each polygon, accumulates edge fan cross products and normalizes the
/// result, matching vtkPolygon::ComputeNormal's handling of polygon cells.
/// Adds a 3-component "FaceNormals" array to cell data.
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::normals_from_faces::compute_face_normals`], which
/// uses VTK's `"Normals"` array name; this entry point keeps the
/// `"FaceNormals"` name it has always produced.
pub fn compute_face_normals(input: &PolyData) -> PolyData {
    let mut pd = crate::filters::mesh::normals_from_faces::compute_face_normals(input);
    if let Some(mut arr) = pd.cell_data_mut().remove_array("Normals") {
        arr.set_name("FaceNormals");
        pd.cell_data_mut().add_array(arr);
    }
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

    #[test]
    fn skips_initial_collinear_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = compute_face_normals(&pd);
        let arr = result.cell_data().get_array("FaceNormals").unwrap();
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!(val[2] > 0.99, "expected +z normal, got {:?}", val);
    }
}
