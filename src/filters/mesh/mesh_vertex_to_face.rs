//! Interpolate vertex scalar data to face (cell) data by averaging.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn vertex_to_face(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    let mut face_vals = Vec::new();
    for cell in mesh.polys.iter() {
        let avg: f64 = cell
            .iter()
            .filter_map(|&v| {
                let vi = v as usize;
                if vi < n {
                    Some(vals[vi])
                } else {
                    None
                }
            })
            .sum::<f64>()
            / cell.len().max(1) as f64;
        face_vals.push(avg);
    }
    let mut result = mesh.clone();
    let out_name = format!("{}_cell", scalar_name);
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &out_name, face_vals, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_v2f() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let r = vertex_to_face(&mesh, "v");
        let arr = r.cell_data().get_array("v_cell").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 2.0).abs() < 1e-9);
    }
}
