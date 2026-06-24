//! Compute vertex normals from face normals (area-weighted average).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute vertex normals by area-weighted averaging of incident face normals.
pub fn compute_vertex_normals(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut normals = vec![[0.0f64; 3]; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ]; // not normalized = area-weighted
            for &v in cell {
                let vi = v as usize;
                if vi < n {
                    normals[vi][0] += fn_[0];
                    normals[vi][1] += fn_[1];
                    normals[vi][2] += fn_[2];
                }
            }
        }
    }

    // Normalize
    for nm in &mut normals {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-15 {
            nm[0] /= len;
            nm[1] /= len;
            nm[2] /= len;
        }
    }

    let data: Vec<f64> = normals.iter().flat_map(|n| n.iter().copied()).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

/// Compute per-face normals (one normal per polygon).
pub fn compute_face_normals(mesh: &PolyData) -> PolyData {
    let mut normals = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let mut n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
        normals.extend_from_slice(&n);
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vertex_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = compute_vertex_normals(&mesh);
        let arr = r.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-10 || (buf[2] + 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_face_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = compute_face_normals(&mesh);
        let arr = r.cell_data().get_array("Normals").unwrap();
        assert_eq!(arr.num_tuples(), 1);
        assert_eq!(arr.num_components(), 3);
    }
}
