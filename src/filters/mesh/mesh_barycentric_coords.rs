//! Compute barycentric coordinates of each vertex relative to its first adjacent triangle.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn barycentric_coords(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut vert_tri: Vec<Option<[usize; 3]>> = vec![None; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let tri = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        for &v in &tri {
            if v < n && vert_tri[v].is_none() {
                vert_tri[v] = Some(tri);
            }
        }
    }
    let mut u_data = vec![0.0f64; n];
    let mut v_data = vec![0.0f64; n];
    let mut w_data = vec![0.0f64; n];
    for i in 0..n {
        if let Some([a, b, c]) = vert_tri[i] {
            if a >= n || b >= n || c >= n {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let p = mesh.points.get(i);
            let v0 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v1 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let v2 = [p[0] - pa[0], p[1] - pa[1], p[2] - pa[2]];
            let d00 = v0[0] * v0[0] + v0[1] * v0[1] + v0[2] * v0[2];
            let d01 = v0[0] * v1[0] + v0[1] * v1[1] + v0[2] * v1[2];
            let d11 = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2];
            let d20 = v2[0] * v0[0] + v2[1] * v0[1] + v2[2] * v0[2];
            let d21 = v2[0] * v1[0] + v2[1] * v1[1] + v2[2] * v1[2];
            let denom = d00 * d11 - d01 * d01;
            if denom.abs() > 1e-15 {
                let bv = (d11 * d20 - d01 * d21) / denom;
                let bw = (d00 * d21 - d01 * d20) / denom;
                let bu = 1.0 - bv - bw;
                u_data[i] = bu;
                v_data[i] = bv;
                w_data[i] = bw;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BaryU", u_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BaryV", v_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BaryW", w_data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = barycentric_coords(&mesh);
        let arr = r.point_data().get_array("BaryU").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-6); // vertex 0 has bary (1,0,0)
    }
}
