//! Interpolate vertex scalar data to face (cell) data by averaging.
use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

pub fn vertex_to_face(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 => a,
        None => return mesh.clone(),
        _ => return mesh.clone(),
    };
    let nt = n.min(arr.num_tuples());
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    let mut face_vals = Vec::with_capacity(mesh.total_cells());
    append_cell_averages(&mesh.verts, nt, &vals, &mut face_vals);
    append_cell_averages(&mesh.lines, nt, &vals, &mut face_vals);
    append_cell_averages(&mesh.polys, nt, &vals, &mut face_vals);
    append_cell_averages(&mesh.strips, nt, &vals, &mut face_vals);
    let mut result = mesh.clone();
    let out_name = format!("{}_cell", scalar_name);
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &out_name, face_vals, 1,
        )));
    result.cell_data_mut().set_active_scalars(&out_name);
    result
}

fn append_cell_averages(cells: &CellArray, nt: usize, vals: &[f64], face_vals: &mut Vec<f64>) {
    for cell in cells.iter() {
        let mut count = 0usize;
        let sum: f64 = cell
            .iter()
            .filter_map(|&v| {
                let vi = usize::try_from(v).ok()?;
                if vi < nt {
                    count += 1;
                    Some(vals[vi])
                } else {
                    None
                }
            })
            .sum();
        face_vals.push(if count > 0 { sum / count as f64 } else { 0.0 });
    }
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
