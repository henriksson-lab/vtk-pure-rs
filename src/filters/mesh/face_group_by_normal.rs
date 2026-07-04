use crate::data::{AnyDataArray, DataArray, PolyData};

/// Group faces by their normal direction.
///
/// Faces whose normals are within `angle_tolerance_deg` of each other belong
/// to the same group. Adds a 1-component "NormalGroup" cell data array with
/// integer group IDs.
///
/// Uses a greedy algorithm: iterates through faces, assigning each to the first
/// existing group whose representative normal is within the tolerance angle,
/// or creating a new group if none match.
pub fn group_faces_by_normal(input: &PolyData, angle_tolerance_deg: f64) -> PolyData {
    let cos_tol: f64 = angle_tolerance_deg.to_radians().cos();

    // Compute face normals
    let face_normals = compute_face_normals(input);
    let num_faces: usize = face_normals.len();

    let mut group_ids: Vec<f64> = Vec::with_capacity(num_faces);
    let mut group_representatives: Vec<[f64; 3]> = Vec::new();

    for i in 0..num_faces {
        let n = &face_normals[i];
        let mut assigned: bool = false;

        for (gid, rep) in group_representatives.iter().enumerate() {
            let dot: f64 = n[0] * rep[0] + n[1] * rep[1] + n[2] * rep[2];
            if dot >= cos_tol {
                group_ids.push(gid as f64);
                assigned = true;
                break;
            }
        }

        if !assigned {
            group_ids.push(group_representatives.len() as f64);
            group_representatives.push(*n);
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalGroup",
            group_ids,
            1,
        )));
    pd
}

fn compute_face_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let mut normals: Vec<[f64; 3]> = Vec::new();

    for cell in input.polys.iter() {
        normals.push(compute_polygon_normal(input, cell));
    }

    normals
}

fn compute_polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let Some(p0_id) = valid_point_index(cell[point_id], input.points.len()) else {
            return [0.0; 3];
        };
        let Some(p1_id) = valid_point_index(cell[point_id + 1], input.points.len()) else {
            return [0.0; 3];
        };
        let p0 = input.points.get(p0_id);
        let p1 = input.points.get(p1_id);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if squared_norm(v1) > 0.0 {
            common = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common else {
        return [0.0; 3];
    };
    if point_id >= cell.len() {
        return [0.0; 3];
    }

    let Some(p0_id) = valid_point_index(cell[common_id], input.points.len()) else {
        return [0.0; 3];
    };
    let p0 = input.points.get(p0_id);
    let mut n = [0.0; 3];
    while point_id < cell.len() {
        let Some(pid) = valid_point_index(cell[point_id], input.points.len()) else {
            return [0.0; 3];
        };
        let p = input.points.get(pid);
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
        [0.0; 3]
    }
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn squared_norm(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coplanar_faces_same_group() {
        // Two triangles in the same plane should be in the same group
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = group_faces_by_normal(&pd, 5.0);
        let arr = result.cell_data().get_array("NormalGroup").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        let mut g0 = [0.0f64];
        let mut g1 = [0.0f64];
        arr.tuple_as_f64(0, &mut g0);
        arr.tuple_as_f64(1, &mut g1);
        assert!(
            (g0[0] - g1[0]).abs() < 1e-10,
            "coplanar faces should share a group"
        );
    }

    #[test]
    fn perpendicular_faces_different_groups() {
        // Two triangles with perpendicular normals (XY plane vs XZ plane)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0], // normal +Z
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 0.0, 1.0], // normal +Y (approx)
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = group_faces_by_normal(&pd, 10.0);
        let arr = result.cell_data().get_array("NormalGroup").unwrap();
        let mut g0 = [0.0f64];
        let mut g1 = [0.0f64];
        arr.tuple_as_f64(0, &mut g0);
        arr.tuple_as_f64(1, &mut g1);
        assert!(
            (g0[0] - g1[0]).abs() > 0.5,
            "perpendicular faces should be in different groups"
        );
    }

    #[test]
    fn wide_tolerance_merges_all() {
        // With 180 degree tolerance, everything should be in group 0
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        );
        let result = group_faces_by_normal(&pd, 180.0);
        let arr = result.cell_data().get_array("NormalGroup").unwrap();
        let mut val = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            assert!(
                val[0].abs() < 1e-10,
                "all faces should be group 0 with 180 deg tolerance"
            );
        }
    }

    #[test]
    fn skips_initial_collinear_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([4.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([5.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.polys.push_cell(&[4, 5, 6, 7]);

        let result = group_faces_by_normal(&pd, 5.0);
        let arr = result.cell_data().get_array("NormalGroup").unwrap();
        let mut g0 = [0.0f64];
        let mut g1 = [0.0f64];
        arr.tuple_as_f64(0, &mut g0);
        arr.tuple_as_f64(1, &mut g1);
        assert!(
            (g0[0] - g1[0]).abs() < 1e-10,
            "collinear-leading coplanar faces should share a group"
        );
    }

    #[test]
    fn invalid_polygon_ids_do_not_panic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = group_faces_by_normal(&pd, 10.0);
        let arr = result.cell_data().get_array("NormalGroup").unwrap();
        assert_eq!(arr.num_tuples(), 3);
    }
}
