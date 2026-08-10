//! Flatten mesh to 2D for texture mapping (Tutte embedding).
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Tutte embedding published as a 2-component "UV" texture coordinate array.
///
/// Thin wrapper over [`crate::filters::mesh::mesh_tutte_embedding::tutte_embedding`],
/// which owns the single implementation and emits the separate "U" and "V" arrays.
pub fn tutte_embedding(mesh: &PolyData, iterations: usize) -> PolyData {
    let mut result = crate::filters::mesh::mesh_tutte_embedding::tutte_embedding(mesh, iterations);
    let Some(u) = result.point_data_mut().remove_array("U") else {
        return result;
    };
    let Some(v) = result.point_data_mut().remove_array("V") else {
        return result;
    };
    let u = u.to_f64_vec();
    let v = v.to_f64_vec();
    let mut uv = Vec::with_capacity(u.len() * 2);
    for (a, b) in u.iter().zip(v.iter()) {
        uv.push(*a);
        uv.push(*b);
    }
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uv, 2)));
    result.point_data_mut().set_active_tcoords("UV");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = tutte_embedding(&m, 50);
        assert!(r.point_data().get_array("UV").is_some());
        assert_eq!(r.point_data().get_array("UV").unwrap().num_components(), 2);
    }
}
