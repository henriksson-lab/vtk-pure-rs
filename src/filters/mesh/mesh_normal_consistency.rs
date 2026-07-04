//! Check face winding consistency and report percentage of consistently oriented faces.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn normal_consistency(mesh: &PolyData) -> (f64, PolyData) {
    let tris: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nt = tris.len();
    if nt < 2 {
        let mut result = mesh.clone();
        result
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "WindingOK",
                vec![1.0; nt],
                1,
            )));
        return (1.0, result);
    }
    // Build directed-edge to face map
    let mut edge_face: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        let nc = tri.len();
        if nc == 0 {
            continue;
        }
        for i in 0..nc {
            edge_face
                .entry((tri[i], tri[(i + 1) % nc]))
                .or_default()
                .push(fi);
        }
    }

    let mut face_ok = vec![1.0f64; nt];
    for cells in edge_face.values() {
        if cells.len() > 1 {
            for &fi in cells {
                face_ok[fi] = 0.0;
            }
        }
    }
    let ratio = face_ok.iter().sum::<f64>() / nt as f64;
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WindingOK",
            face_ok,
            1,
        )));
    (ratio, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_consistency() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]], // consistent winding
        );
        let (ratio, _) = normal_consistency(&mesh);
        assert!(ratio > 0.5);
    }

    #[test]
    fn single_cell_adds_winding_array() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (ratio, result) = normal_consistency(&mesh);
        assert_eq!(ratio, 1.0);
        let arr = result.cell_data().get_array("WindingOK").unwrap();
        assert_eq!(arr.num_tuples(), 1);
    }
}
