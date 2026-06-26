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
        if cell.len() < 3 {
            normals.push([0.0, 0.0, 0.0]);
            continue;
        }

        // Newell's method
        let mut nx: f64 = 0.0;
        let mut ny: f64 = 0.0;
        let mut nz: f64 = 0.0;
        let n: usize = cell.len();
        for j in 0..n {
            let p = input.points.get(cell[j] as usize);
            let q = input.points.get(cell[(j + 1) % n] as usize);
            nx += (p[1] - q[1]) * (p[2] + q[2]);
            ny += (p[2] - q[2]) * (p[0] + q[0]);
            nz += (p[0] - q[0]) * (p[1] + q[1]);
        }
        let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-20 {
            normals.push([nx / len, ny / len, nz / len]);
        } else {
            normals.push([0.0, 0.0, 0.0]);
        }
    }

    normals
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
}
