//! Compute and store face normals as cell data (3-component).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_normals(mesh: &PolyData) -> PolyData {
    let mut normals = Vec::new();
    for cell in mesh.polys.iter() {
        normals.extend_from_slice(&polygon_normal(mesh, cell));
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    result.cell_data_mut().set_active_normals("Normals");
    result
}

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 0.0];
    }

    let mut common_id = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let Some(p0) = point(mesh, cell[point_id]) else {
            return [0.0, 0.0, 0.0];
        };
        let Some(p1) = point(mesh, cell[point_id + 1]) else {
            return [0.0, 0.0, 0.0];
        };
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if squared_norm(v1) > 0.0 {
            common_id = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common_id else {
        return [0.0, 0.0, 0.0];
    };
    let Some(p0) = point(mesh, cell[common_id]) else {
        return [0.0, 0.0, 0.0];
    };

    let mut n = [0.0; 3];
    while point_id < cell.len() {
        let Some(p) = point(mesh, cell[point_id]) else {
            return [0.0, 0.0, 0.0];
        };
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

fn point(mesh: &PolyData, id: i64) -> Option<[f64; 3]> {
    usize::try_from(id)
        .ok()
        .filter(|&idx| idx < mesh.points.len())
        .map(|idx| mesh.points.get(idx))
}

fn squared_norm(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_face_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_normals(&mesh);
        let normals = r.cell_data().get_array("Normals").unwrap();
        assert_eq!(normals.num_components(), 3);
        let mut b = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut b);
        assert!((b[2] - 1.0).abs() < 1e-6);
    }
}
