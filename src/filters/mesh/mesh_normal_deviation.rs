//! Compute deviation angle between vertex normal and adjacent face normals.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn normal_deviation(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let mut vertex_normals = vec![[0.0f64; 3]; n];
    let mut face_normals_per_vertex: Vec<Vec<[f64; 3]>> = vec![Vec::new(); n];

    for cell in mesh.polys.iter() {
        if cell.iter().any(|&point_id| point(mesh, point_id).is_none()) {
            continue;
        }
        let face_normal = polygon_normal(mesh, cell);
        let length = norm(face_normal);
        if length == 0.0 {
            continue;
        }
        let unit_face_normal = [
            face_normal[0] / length,
            face_normal[1] / length,
            face_normal[2] / length,
        ];

        for &point_id in cell {
            let point_id = point_id as usize;
            vertex_normals[point_id][0] += face_normal[0];
            vertex_normals[point_id][1] += face_normal[1];
            vertex_normals[point_id][2] += face_normal[2];
            face_normals_per_vertex[point_id].push(unit_face_normal);
        }
    }

    for normal in &mut vertex_normals {
        normalize(normal);
    }

    let deviation: Vec<f64> = (0..n)
        .map(|i| {
            let vertex_normal = vertex_normals[i];
            let mut max_angle = 0.0f64;
            for face_normal in &face_normals_per_vertex[i] {
                let dot = (vertex_normal[0] * face_normal[0]
                    + vertex_normal[1] * face_normal[1]
                    + vertex_normal[2] * face_normal[2])
                    .clamp(-1.0, 1.0);
                max_angle = max_angle.max(dot.acos().to_degrees());
            }
            max_angle
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalDeviation",
            deviation,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("NormalDeviation");
    result
}

fn polygon_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut common_id = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let Some(p0) = point(mesh, cell[point_id]) else {
            return [0.0; 3];
        };
        let Some(p1) = point(mesh, cell[point_id + 1]) else {
            return [0.0; 3];
        };
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if norm_squared(v1) > 0.0 {
            common_id = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common_id else {
        return [0.0; 3];
    };
    let Some(p0) = point(mesh, cell[common_id]) else {
        return [0.0; 3];
    };

    let mut normal = [0.0; 3];
    while point_id < cell.len() {
        let Some(p) = point(mesh, cell[point_id]) else {
            return [0.0; 3];
        };
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        normal[0] += cross[0];
        normal[1] += cross[1];
        normal[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    normal
}

fn point(mesh: &PolyData, id: i64) -> Option<[f64; 3]> {
    usize::try_from(id)
        .ok()
        .filter(|&idx| idx < mesh.points.len())
        .map(|idx| mesh.points.get(idx))
}

fn normalize(vector: &mut [f64; 3]) {
    let length = norm(*vector);
    if length > 0.0 {
        vector[0] /= length;
        vector[1] /= length;
        vector[2] /= length;
    }
}

fn norm(vector: [f64; 3]) -> f64 {
    norm_squared(vector).sqrt()
}

fn norm_squared(vector: [f64; 3]) -> f64 {
    vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deviation() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = normal_deviation(&mesh);
        assert!(r.point_data().get_array("NormalDeviation").is_some());
    }
}
