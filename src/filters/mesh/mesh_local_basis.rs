//! Compute orthonormal local basis (tangent, bitangent, normal) per vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn local_basis(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut vnorm = vec![[0.0f64; 3]; n];
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
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                vnorm[vi][0] += nx;
                vnorm[vi][1] += ny;
                vnorm[vi][2] += nz;
            }
        }
    }
    let mut tx = vec![0.0f64; n];
    let mut ty = vec![0.0f64; n];
    let mut tz = vec![0.0f64; n];
    let mut bx = vec![0.0f64; n];
    let mut by = vec![0.0f64; n];
    let mut bz = vec![0.0f64; n];
    let mut nnx = vec![0.0f64; n];
    let mut nny = vec![0.0f64; n];
    let mut nnz = vec![0.0f64; n];
    for i in 0..n {
        let nn = vnorm[i];
        let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
        let normal = if l > 1e-15 {
            [nn[0] / l, nn[1] / l, nn[2] / l]
        } else {
            [0.0, 0.0, 1.0]
        };
        nnx[i] = normal[0];
        nny[i] = normal[1];
        nnz[i] = normal[2];
        // Build tangent: pick axis least aligned with normal
        let up = if normal[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let t = [
            up[1] * normal[2] - up[2] * normal[1],
            up[2] * normal[0] - up[0] * normal[2],
            up[0] * normal[1] - up[1] * normal[0],
        ];
        let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        if tl > 1e-15 {
            tx[i] = t[0] / tl;
            ty[i] = t[1] / tl;
            tz[i] = t[2] / tl;
        }
        // Bitangent = normal x tangent
        bx[i] = normal[1] * tz[i] - normal[2] * ty[i];
        by[i] = normal[2] * tx[i] - normal[0] * tz[i];
        bz[i] = normal[0] * ty[i] - normal[1] * tx[i];
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TangentX", tx, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TangentY", ty, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TangentZ", tz, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BitangentX", bx, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BitangentY", by, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("BitangentZ", bz, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("NormalX", nnx, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("NormalY", nny, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("NormalZ", nnz, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basis() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = local_basis(&mesh);
        assert!(r.point_data().get_array("TangentX").is_some());
        assert!(r.point_data().get_array("NormalZ").is_some());
        // Normal should be [0,0,1] for flat XY triangle
        let arr = r.point_data().get_array("NormalZ").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-6);
    }
}
