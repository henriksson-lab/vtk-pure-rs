//! Compute vertex normals from face normals.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute vertex normals by averaging incident face normals.
pub fn compute_vertex_normals(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut normals = vec![[0.0f64; 3]; n];

    for cell in mesh.polys.iter() {
        let face_normal = polygon_normal(mesh, cell);
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                normals[vi][0] += face_normal[0];
                normals[vi][1] += face_normal[1];
                normals[vi][2] += face_normal[2];
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
        let n = polygon_normal(mesh, cell);
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

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 0.0];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let p0 = mesh.points.get(cell[point_id] as usize);
        let p1 = mesh.points.get(cell[point_id + 1] as usize);
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

    let p0 = mesh.points.get(cell[common_id] as usize);
    let mut n = [0.0; 3];
    while point_id < cell.len() {
        let p = mesh.points.get(cell[point_id] as usize);
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

    #[test]
    fn face_normals_skip_initial_collinear_vertices() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.points.push([2.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let r = compute_face_normals(&mesh);
        let arr = r.cell_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.99, "expected +z normal, got {:?}", buf);
    }

    #[test]
    fn degenerate_face_normal_is_zero() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);

        let r = compute_face_normals(&mesh);
        let arr = r.cell_data().get_array("Normals").unwrap();
        let mut buf = [1.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 0.0]);
    }
}
