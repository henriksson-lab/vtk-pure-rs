//! Area-weighted vertex normals.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn area_weighted_normals(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    if mesh.verts.iter().next().is_some()
        || mesh.lines.iter().next().is_some()
        || mesh.strips.iter().next().is_some()
    {
        return mesh.clone();
    }
    if mesh.polys.iter().next().is_none() {
        return mesh.clone();
    }
    if mesh.polys.iter().any(|cell| cell.len() != 3) {
        return mesh.clone();
    }
    if mesh
        .polys
        .iter()
        .any(|cell| cell.iter().any(|&id| id < 0 || (id as usize) >= n))
    {
        return mesh.clone();
    }

    let mut normals = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        let p0 = mesh.points.get(cell[0] as usize);
        let p1 = mesh.points.get(cell[1] as usize);
        let p2 = mesh.points.get(cell[2] as usize);
        let a = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let b = [p0[0] - p1[0], p0[1] - p1[1], p0[2] - p1[2]];
        let tri_normal = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        for &id in cell {
            let idx = id as usize;
            normals[idx][0] += tri_normal[0];
            normals[idx][1] += tri_normal[1];
            normals[idx][2] += tri_normal[2];
        }
    }

    let flat: Vec<f64> = normals
        .iter()
        .flat_map(|vn| {
            let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
            if len > 1e-15 {
                vec![vn[0] / len, vn[1] / len, vn[2] / len]
            } else {
                vec![0.0, 0.0, 0.0]
            }
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", flat, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_area_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = area_weighted_normals(&mesh);
        assert!(r.point_data().get_array("Normals").is_some());
        let arr = r.point_data().get_array("Normals").unwrap();
        let mut b = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[2] - 1.0).abs() < 1e-6); // flat XY triangle => normal is Z
    }

    #[test]
    fn test_area_normals_rejects_non_triangles() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let r = area_weighted_normals(&mesh);
        assert!(r.point_data().get_array("Normals").is_none());
    }
}
