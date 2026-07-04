//! Simple vertex normal computation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn compute_vertex_normals_simple(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || cell.iter().any(|&v| valid_point_id(v, n).is_none()) {
            continue;
        }
        let mut fn_ = polygon_normal_raw(mesh, cell);
        normalize(&mut fn_);
        for &v in cell {
            let vi = v as usize;
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
    let data: Vec<f64> = nm.iter().flat_map(|n| n.iter().copied()).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    r.point_data_mut().set_active_normals("Normals");
    r
}
pub fn compute_cell_normals_simple(mesh: &PolyData) -> PolyData {
    let mut data = Vec::new();
    for _ in mesh.verts.iter() {
        data.extend_from_slice(&[1.0, 0.0, 0.0]);
    }
    for _ in mesh.lines.iter() {
        data.extend_from_slice(&[1.0, 0.0, 0.0]);
    }
    for cell in mesh.polys.iter() {
        if cell.len() < 3
            || cell
                .iter()
                .any(|&v| valid_point_id(v, mesh.points.len()).is_none())
        {
            data.extend_from_slice(&[0.0, 0.0, 0.0]);
            continue;
        }
        let mut n = polygon_normal_raw(mesh, cell);
        normalize(&mut n);
        data.extend_from_slice(&n);
    }
    for _ in mesh.strips.iter() {
        data.extend_from_slice(&[1.0, 0.0, 0.0]);
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    r.cell_data_mut().set_active_normals("Normals");
    r
}
fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < n)
}

fn polygon_normal_raw(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut point_id = 0usize;
    let mut common_point_id = None;
    let mut v1 = [0.0; 3];

    while point_id < cell.len().saturating_sub(2) {
        let p0 = mesh.points.get(cell[point_id] as usize);
        let p1 = mesh.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2] > 0.0 {
            common_point_id = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_point_id) = common_point_id else {
        return [0.0; 3];
    };
    if point_id >= cell.len() {
        return [0.0; 3];
    }

    let p0 = mesh.points.get(cell[common_point_id] as usize);
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
    n
}

fn normalize(n: &mut [f64; 3]) {
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if length != 0.0 {
        n[0] /= length;
        n[1] /= length;
        n[2] /= length;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vert() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = compute_vertex_normals_simple(&m);
        let arr = r.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2].abs() > 0.9);
    }
    #[test]
    fn test_cell() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = compute_cell_normals_simple(&m);
        assert!(r.cell_data().get_array("Normals").is_some());
    }

    #[test]
    fn cell_normals_match_polydata_cell_order() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.points.push([0.0, 0.0, 1.0]);
        m.verts.push_cell(&[0]);
        m.lines.push_cell(&[0, 1]);
        m.polys.push_cell(&[0, 1, 2]);
        m.strips.push_cell(&[0, 1, 2, 3]);

        let r = compute_cell_normals_simple(&m);
        let arr = r.cell_data().get_array("Normals").unwrap();

        assert_eq!(arr.num_tuples(), m.total_cells());
        let mut first = [0.0; 3];
        let mut second = [0.0; 3];
        arr.tuple_as_f64(0, &mut first);
        arr.tuple_as_f64(1, &mut second);
        assert_eq!(first, [1.0, 0.0, 0.0]);
        assert_eq!(second, [1.0, 0.0, 0.0]);
    }
}
