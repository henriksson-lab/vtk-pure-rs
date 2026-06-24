//! Smooth vertex normals by averaging face normals weighted by face area.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn smooth_normals(mesh: &PolyData) -> PolyData {
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
        // Area-weighted normal (cross product, not normalized)
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
    // Normalize
    let mut nx_data = Vec::with_capacity(n * 3);
    for vn in &vnorm {
        let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if len > 1e-15 {
            nx_data.push(vn[0] / len);
            nx_data.push(vn[1] / len);
            nx_data.push(vn[2] / len);
        } else {
            nx_data.push(0.0);
            nx_data.push(0.0);
            nx_data.push(1.0);
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", nx_data, 3,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_smooth_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = smooth_normals(&mesh);
        let arr = r.point_data().get_array("Normals").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut b = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[2] - 1.0).abs() < 1e-6); // Z normal for flat XY tri
    }
}
