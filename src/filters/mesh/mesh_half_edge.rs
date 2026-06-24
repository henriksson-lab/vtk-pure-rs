//! Build half-edge connectivity and export boundary/manifold info as scalars.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn half_edge_analysis(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Count directed edges
    let mut directed: std::collections::HashMap<(i64, i64), u32> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            *directed.entry((a, b)).or_insert(0) += 1;
        }
    }
    // Classify vertices
    let mut boundary = vec![0.0f64; n];
    let mut non_manifold = vec![0.0f64; n];
    for (&(a, b), &count) in &directed {
        // Check if twin exists
        let twin_count = directed.get(&(b, a)).copied().unwrap_or(0);
        if twin_count == 0 {
            // Boundary edge
            boundary[a as usize] = 1.0;
            boundary[b as usize] = 1.0;
        }
        if count > 1 || twin_count > 1 {
            // Non-manifold edge
            non_manifold[a as usize] = 1.0;
            non_manifold[b as usize] = 1.0;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Boundary", boundary, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NonManifold",
            non_manifold,
            1,
        )));
    result.point_data_mut().set_active_scalars("Boundary");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_half_edge() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = half_edge_analysis(&mesh);
        // All vertices are boundary (single triangle)
        let arr = r.point_data().get_array("Boundary").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 1.0);
    }
}
