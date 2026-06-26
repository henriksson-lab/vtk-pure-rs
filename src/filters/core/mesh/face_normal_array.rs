use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-face (cell) normals via cross product and add as cell data.
///
/// For each polygon, accumulates edge fan cross products and normalizes the
/// result, matching vtkPolygon::ComputeNormal's handling of polygon cells.
/// Adds a 3-component "FaceNormals" array to cell data.
pub fn compute_face_normals(input: &PolyData) -> PolyData {
    let mut normals: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        let normal = polygon_normal(input, cell);
        normals.extend_from_slice(&normal);
    }

    let mut pd = input.clone();
    pd.cell_data_mut().add_array(AnyDataArray::F64(
        DataArray::from_vec("FaceNormals", normals, 3),
    ));
    pd
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 0.0];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let p0 = input.points.get(cell[point_id] as usize);
        let p1 = input.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if squared_norm(v1) > 0.0 {
            common = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common else {
        return [0.0, 0.0, 0.0];
    };
    if point_id >= cell.len() {
        return [0.0, 0.0, 0.0];
    }

    let p0 = input.points.get(cell[common_id] as usize);
    let mut n = [0.0; 3];
    while point_id < cell.len() {
        let p = input.points.get(cell[point_id] as usize);
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        n[0] += cross[0];
        n[1] += cross[1];
        n[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    let len = squared_norm(n).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn squared_norm(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
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
