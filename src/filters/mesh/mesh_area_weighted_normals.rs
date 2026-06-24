//! Area-weighted vertex normals.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn area_weighted_normals(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut normals = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        // Cross product (NOT normalized = area-weighted)
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                normals[vi][0] += nx;
                normals[vi][1] += ny;
                normals[vi][2] += nz;
            }
        }
    }
    // Normalize
    let mut nx_data = Vec::with_capacity(n);
    let mut ny_data = Vec::with_capacity(n);
    let mut nz_data = Vec::with_capacity(n);
    for vn in &normals {
        let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if len > 1e-15 {
            nx_data.push(vn[0] / len);
            ny_data.push(vn[1] / len);
            nz_data.push(vn[2] / len);
        } else {
            nx_data.push(0.0);
            ny_data.push(0.0);
            nz_data.push(1.0);
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalX", nx_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalY", ny_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalZ", nz_data, 1,
        )));
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
        assert!(r.point_data().get_array("NormalZ").is_some());
        let arr = r.point_data().get_array("NormalZ").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-6); // flat XY triangle => normal is Z
    }
}
