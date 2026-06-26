use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute deviation of vertex normals from face normals.
///
/// For each vertex, measures how much its smooth normal deviates from
/// the adjacent face normals. High deviation = sharp feature.
/// Adds "NormalDeviation" scalar (angle in degrees).
pub fn normal_deviation(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut vertex_normals = vec![[0.0f64; 3]; n];
    let mut face_normals_per_vertex: Vec<Vec<[f64; 3]>> = vec![Vec::new(); n];

    for cell in input.polys.iter() {
        let face_normal = polygon_normal(input, cell);
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
            if point_id >= n {
                continue;
            }
            vertex_normals[point_id][0] += face_normal[0];
            vertex_normals[point_id][1] += face_normal[1];
            vertex_normals[point_id][2] += face_normal[2];
            face_normals_per_vertex[point_id].push(unit_face_normal);
        }
    }

    for normal in &mut vertex_normals {
        normalize(normal);
    }

    let mut deviation = vec![0.0f64; n];
    for i in 0..n {
        let vertex_normal = vertex_normals[i];
        let mut max_angle = 0.0f64;
        for face_normal in &face_normals_per_vertex[i] {
            let dot = (vertex_normal[0] * face_normal[0]
                + vertex_normal[1] * face_normal[1]
                + vertex_normal[2] * face_normal[2])
                .clamp(-1.0, 1.0);
            let angle = dot.acos().to_degrees();
            max_angle = max_angle.max(angle);
        }
        deviation[i] = max_angle;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalDeviation",
            deviation,
            1,
        )));
    pd.point_data_mut().set_active_scalars("NormalDeviation");
    pd
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let p0 = input.points.get(cell[point_id] as usize);
        let p1 = input.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if norm_squared(v1) > 0.0 {
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

    let p0 = input.points.get(cell[common_id] as usize);
    let mut normal = [0.0; 3];
    while point_id < cell.len() {
        let p = input.points.get(cell[point_id] as usize);
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
    fn flat_zero_deviation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = normal_deviation(&pd);
        let arr = result.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0f64];
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 5.0);
        }
        assert!(result.point_data().scalars().is_some());
    }

    #[test]
    fn sharp_high_deviation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = normal_deviation(&pd);
        let arr = result.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 10.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = normal_deviation(&pd);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn polygon_normal_skips_initial_collinear_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3, 4]);

        let result = normal_deviation(&pd);
        let arr = result.point_data().get_array("NormalDeviation").unwrap();
        assert_eq!(arr.num_tuples(), 5);
    }
}
