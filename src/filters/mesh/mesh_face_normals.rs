//! Compute and store face normals as cell data (3-component).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_normals(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut nx_data = Vec::new();
    let mut ny_data = Vec::new();
    let mut nz_data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            nx_data.push(0.0);
            ny_data.push(0.0);
            nz_data.push(1.0);
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            nx_data.push(0.0);
            ny_data.push(0.0);
            nz_data.push(1.0);
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let fnx = u[1] * v[2] - u[2] * v[1];
        let fny = u[2] * v[0] - u[0] * v[2];
        let fnz = u[0] * v[1] - u[1] * v[0];
        let len = (fnx * fnx + fny * fny + fnz * fnz).sqrt();
        if len > 1e-15 {
            nx_data.push(fnx / len);
            ny_data.push(fny / len);
            nz_data.push(fnz / len);
        } else {
            nx_data.push(0.0);
            ny_data.push(0.0);
            nz_data.push(1.0);
        }
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceNormalX",
            nx_data,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceNormalY",
            ny_data,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceNormalZ",
            nz_data,
            1,
        )));
    result
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
        let nz = r.cell_data().get_array("FaceNormalZ").unwrap();
        let mut b = [0.0f64];
        nz.tuple_as_f64(0, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-6);
    }
}
