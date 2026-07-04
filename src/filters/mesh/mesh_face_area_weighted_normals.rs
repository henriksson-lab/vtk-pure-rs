//! Compute face-area-weighted vertex normals.
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

    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        let p0 = mesh.points.get(cell[0] as usize);
        let p1 = mesh.points.get(cell[1] as usize);
        let p2 = mesh.points.get(cell[2] as usize);
        let e1 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let e2 = [p0[0] - p1[0], p0[1] - p1[1], p0[2] - p1[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = area_weighted_normals(&m);
        let arr = r.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2].abs() > 0.9);
    }
}
