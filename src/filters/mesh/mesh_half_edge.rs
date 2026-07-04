//! Build half-edge connectivity and export boundary/manifold info as scalars.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn half_edge_analysis(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Count polygon use per undirected edge.
    let mut edges: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    // Classify vertices
    let mut boundary = vec![0.0f64; n];
    let mut non_manifold = vec![0.0f64; n];
    for (&(a, b), &count) in &edges {
        if count == 1 {
            boundary[a] = 1.0;
            boundary[b] = 1.0;
        }
        if count > 2 {
            non_manifold[a] = 1.0;
            non_manifold[b] = 1.0;
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

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
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
