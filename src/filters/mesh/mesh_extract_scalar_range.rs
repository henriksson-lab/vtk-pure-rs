//! Extract faces whose vertex scalar values fall within a range.
use crate::data::{CellArray, PolyData};

pub fn extract_scalar_range(
    mesh: &PolyData,
    scalar_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
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
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let all_in = cell.iter().all(|&v| {
            let vi = v as usize;
            vi < n && vals[vi] >= min_val && vals[vi] <= max_val
        });
        if all_in {
            polys.push_cell(&cell.to_vec());
        }
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_extract() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 0.3, 1.0],
                1,
            )));
        let r = extract_scalar_range(&mesh, "v", 0.0, 0.6);
        assert_eq!(r.polys.num_cells(), 1); // only first face (all verts <= 0.6)
    }
}
