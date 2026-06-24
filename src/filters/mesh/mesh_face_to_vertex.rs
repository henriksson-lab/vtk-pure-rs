//! Interpolate cell (face) data to vertex (point) data by averaging incident faces.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_to_vertex(mesh: &PolyData, cell_scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.cell_data().get_array(cell_scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut sum = vec![0.0f64; n];
    let mut count = vec![0u32; n];
    let mut buf = [0.0f64];
    for (fi, cell) in mesh.polys.iter().enumerate() {
        arr.tuple_as_f64(fi, &mut buf);
        for &v in &cell[..] {
            let vi = v as usize;
            if vi < n {
                sum[vi] += buf[0];
                count[vi] += 1;
            }
        }
    }
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            if count[i] > 0 {
                sum[i] / count[i] as f64
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    let out_name = format!("{}_point", cell_scalar_name);
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out_name, vals, 1)));
    result.point_data_mut().set_active_scalars(&out_name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_f2v() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cv",
                vec![10.0, 20.0],
                1,
            )));
        let r = face_to_vertex(&mesh, "cv");
        let arr = r.point_data().get_array("cv_point").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b); // vertex 1 is in both faces
        assert!((b[0] - 15.0).abs() < 1e-9); // average of 10 and 20
    }
}
