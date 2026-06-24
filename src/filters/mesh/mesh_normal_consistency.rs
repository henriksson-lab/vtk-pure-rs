//! Check face winding consistency and report percentage of consistently oriented faces.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn normal_consistency(mesh: &PolyData) -> (f64, PolyData) {
    let tris: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nt = tris.len();
    if nt < 2 {
        return (1.0, mesh.clone());
    }
    // Build directed-edge to face map
    let mut edge_face: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        let nc = tri.len();
        for i in 0..nc {
            edge_face.insert((tri[i], tri[(i + 1) % nc]), fi);
        }
    }
    // For each edge, check if the opposite half-edge exists (consistent) or same direction (inconsistent)
    let mut consistent = 0usize;
    let mut total = 0usize;
    let mut face_ok = vec![1.0f64; nt];
    for (fi, tri) in tris.iter().enumerate() {
        let nc = tri.len();
        for i in 0..nc {
            let a = tri[i];
            let b = tri[(i + 1) % nc];
            total += 1;
            if edge_face.contains_key(&(b, a)) {
                consistent += 1;
            } else if edge_face.contains_key(&(a, b)) {
                // Same direction = inconsistent neighbor
                face_ok[fi] = 0.0;
            }
        }
    }
    let ratio = if total > 0 {
        consistent as f64 / total as f64
    } else {
        1.0
    };
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
}
