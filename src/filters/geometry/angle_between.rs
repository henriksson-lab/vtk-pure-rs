use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Compute dihedral angles between adjacent triangles.
///
/// For each edge shared by two triangles, computes the angle between
/// their face normals. Adds a "DihedralAngle" cell data array with
/// the minimum dihedral angle per cell (in degrees).
pub fn dihedral_angles(input: &PolyData) -> PolyData {
    let nc = input.polys.num_cells();
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();

    // Compute face normals from raw connectivity using flat point access
    let pts = input.points.as_flat_slice();
    let mut normals: Vec<[f64; 3]> = Vec::with_capacity(nc);
    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        if end - start < 3 {
            normals.push([0.0, 0.0, 0.0]);
            continue;
        }
        let b0 = conn[start] as usize * 3;
        let b1 = conn[start + 1] as usize * 3;
        let b2 = conn[start + 2] as usize * 3;
        let e1 = [
            pts[b1] - pts[b0],
            pts[b1 + 1] - pts[b0 + 1],
            pts[b1 + 2] - pts[b0 + 2],
        ];
        let e2 = [
            pts[b2] - pts[b0],
            pts[b2 + 1] - pts[b0 + 1],
            pts[b2 + 2] - pts[b0 + 2],
        ];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            normals.push([n[0] / len, n[1] / len, n[2] / len]);
        } else {
            normals.push([0.0, 0.0, 0.0]);
        }
    }

    // Build edge adjacency using packed u64 keys: (face0, face1)
    let mut edge_faces: HashMap<u64, (u32, u32)> = HashMap::with_capacity(nc * 3);
    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let cell = &conn[start..end];
        let n = cell.len();
        for i in 0..n {
            let a = cell[i];
            let b = cell[(i + 1) % n];
            let key = if a < b {
                (a as u64) << 32 | b as u64
            } else {
                (b as u64) << 32 | a as u64
            };
            let entry = edge_faces.entry(key).or_insert((ci as u32, u32::MAX));
            if entry.0 != ci as u32 && entry.1 == u32::MAX {
                entry.1 = ci as u32;
            }
        }
    }

    // For each cell, find minimum dihedral angle with neighbors
    let mut min_angles = vec![180.0f64; nc];
    for &(f0, f1) in edge_faces.values() {
        if f1 == u32::MAX {
            continue;
        }
        let na = normals[f0 as usize];
        let nb = normals[f1 as usize];
        let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
        let angle = dot.acos().to_degrees();
        if angle < min_angles[f0 as usize] {
            min_angles[f0 as usize] = angle;
        }
        if angle < min_angles[f1 as usize] {
            min_angles[f1 as usize] = angle;
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DihedralAngle",
            min_angles,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_surface_zero_angle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = dihedral_angles(&pd);
        let arr = result.cell_data().get_array("DihedralAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1.0);
    }

    #[test]
    fn right_angle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = dihedral_angles(&pd);
        let arr = result.cell_data().get_array("DihedralAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 90.0).abs() < 5.0);
    }

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = dihedral_angles(&pd);
        let arr = result.cell_data().get_array("DihedralAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 180.0);
    }
}
