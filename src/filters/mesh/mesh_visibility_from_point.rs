//! Compute visibility of each face from a viewpoint.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_visibility(mesh: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        let ids = match valid_cell_prefix(cell, mesh.points.len()) {
            Some(ids) => ids,
            None => {
                data.push(0.0);
                continue;
            }
        };
        if ids.len() < 3 {
            data.push(0.0);
            continue;
        }
        let n = polygon_normal(mesh, &ids);
        let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if nl < 1e-15 {
            data.push(0.0);
            continue;
        }
        let centroid = polygon_centroid(mesh, &ids);
        let cx = centroid[0];
        let cy = centroid[1];
        let cz = centroid[2];
        let to_view = [viewpoint[0] - cx, viewpoint[1] - cy, viewpoint[2] - cz];
        let dot = (n[0] * to_view[0] + n[1] * to_view[1] + n[2] * to_view[2]) / nl;
        data.push(if dot > 0.0 { 1.0 } else { 0.0 });
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Visible", data, 1)));
    r
}
pub fn vertex_visibility(mesh: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let to = [
                viewpoint[0] - p[0],
                viewpoint[1] - p[1],
                viewpoint[2] - p[2],
            ];
            let dot = nm[i][0] * to[0] + nm[i][1] * to[1] + nm[i][2] * to[2];
            if dot > 0.0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Visible", data, 1)));
    r
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        let ids = match valid_cell_prefix(cell, n) {
            Some(ids) if ids.len() >= 3 => ids,
            _ => {
                continue;
            }
        };
        let fn_ = polygon_normal(mesh, &ids);
        for &vi in &ids {
            nm[vi][0] += fn_[0];
            nm[vi][1] += fn_[1];
            nm[vi][2] += fn_[2];
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}

fn polygon_centroid(mesh: &PolyData, ids: &[usize]) -> [f64; 3] {
    let mut centroid = [0.0; 3];
    for &id in ids {
        let p = mesh.points.get(id);
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }
    let scale = 1.0 / ids.len() as f64;
    [
        centroid[0] * scale,
        centroid[1] * scale,
        centroid[2] * scale,
    ]
}

fn polygon_normal(mesh: &PolyData, ids: &[usize]) -> [f64; 3] {
    if ids.len() < 3 {
        return [0.0; 3];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < ids.len() - 2 {
        let p0 = mesh.points.get(ids[point_id]);
        let p1 = mesh.points.get(ids[point_id + 1]);
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
    if point_id >= ids.len() {
        return [0.0; 3];
    }

    let p0 = mesh.points.get(ids[common_id]);
    let mut normal = [0.0; 3];
    while point_id < ids.len() {
        let p = mesh.points.get(ids[point_id]);
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

fn norm_squared(vector: [f64; 3]) -> f64 {
    vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]
}

fn valid_cell_prefix(cell: &[i64], num_points: usize) -> Option<Vec<usize>> {
    let mut ids = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= num_points {
            return None;
        }
        ids.push(id as usize);
    }
    Some(ids)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_face() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_visibility(&m, [0.5, 0.5, 10.0]);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Visible")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn test_face_with_initial_collinear_points() {
        let m = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3, 4]],
        );
        let r = face_visibility(&m, [1.0, 0.5, 10.0]);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("Visible")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn test_vertex() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = vertex_visibility(&m, [0.5, 0.5, 10.0]);
        assert!(r.point_data().get_array("Visible").is_some());
    }
}
