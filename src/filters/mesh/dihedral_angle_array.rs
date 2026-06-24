use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Compute dihedral angles for interior edges of a triangle mesh.
///
/// Returns a new PolyData where each cell is a line (edge) and the cell data
/// contains a "DihedralAngle" array with the dihedral angle in degrees.
/// Interior edges (shared by exactly two triangles) get the true dihedral angle.
/// Boundary edges (only one adjacent triangle) are excluded.
pub fn compute_dihedral_angles(input: &PolyData) -> PolyData {
    // Build edge -> [face_index, ...] map
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for (fi, cell) in input.polys.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Collect face normals
    let face_normals = compute_face_normals(input);

    let mut points = Points::default();
    let mut lines = CellArray::default();
    let mut angles: Vec<f64> = Vec::new();

    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue; // skip boundary or non-manifold edges
        }

        let n0 = &face_normals[faces[0]];
        let n1 = &face_normals[faces[1]];

        // Dihedral angle: angle between the two face normals
        let dot: f64 = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
        let clamped: f64 = dot.max(-1.0).min(1.0);
        let angle_deg: f64 = clamped.acos().to_degrees();

        let pa = input.points.get(a);
        let pb = input.points.get(b);

        let idx_a = points.len() as i64;
        points.push(pa);
        points.push(pb);
        lines.push_cell(&[idx_a, idx_a + 1]);

        angles.push(angle_deg);
    }

    let mut pd = PolyData::default();
    pd.points = points;
    pd.lines = lines;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DihedralAngle",
            angles,
            1,
        )));
    pd
}

fn compute_face_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let mut normals: Vec<[f64; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            normals.push([0.0, 0.0, 1.0]);
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);

        let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx: f64 = u[1] * v[2] - u[2] * v[1];
        let ny: f64 = u[2] * v[0] - u[0] * v[2];
        let nz: f64 = u[0] * v[1] - u[1] * v[0];
        let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();

        if len > 1e-20 {
            normals.push([nx / len, ny / len, nz / len]);
        } else {
            normals.push([0.0, 0.0, 1.0]);
        }
    }
    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coplanar_triangles_zero_angle() {
        // Two coplanar triangles sharing edge (1,2)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = compute_dihedral_angles(&pd);
        let arr = result.cell_data().get_array("DihedralAngle").unwrap();
        // All interior edges between coplanar faces should have angle ~ 0 degrees
        for i in 0..arr.num_tuples() {
            let mut val = [0.0f64];
            arr.tuple_as_f64(i, &mut val);
            assert!(val[0] < 1.0, "coplanar angle should be ~0, got {}", val[0]);
        }
    }

    #[test]
    fn right_angle_fold() {
        // Two triangles folded at 90 degrees along the x-axis
        // Triangle 1: lies in z=0 plane
        // Triangle 2: lies in y=0 plane
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = compute_dihedral_angles(&pd);
        let arr = result.cell_data().get_array("DihedralAngle").unwrap();
        // Should find the shared edge with ~90 degree dihedral angle
        let mut found_90 = false;
        for i in 0..arr.num_tuples() {
            let mut val = [0.0f64];
            arr.tuple_as_f64(i, &mut val);
            if (val[0] - 90.0).abs() < 1.0 {
                found_90 = true;
            }
        }
        assert!(found_90, "should find a ~90 degree dihedral angle");
    }

    #[test]
    fn single_triangle_no_interior_edges() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_dihedral_angles(&pd);
        // Single triangle has no interior edges
        assert_eq!(result.lines.num_cells(), 0);
    }
}
