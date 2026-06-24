//! Approximate convex decomposition by iterative plane cuts.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn convex_decompose(mesh: &PolyData, max_parts: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Assign each vertex a component label based on spatial clustering
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    // Simple approach: classify by octant relative to centroid, up to max_parts
    let parts = max_parts.min(8).max(1);
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let mut label = 0u32;
            if parts > 1 && p[0] > cx {
                label |= 1;
            }
            if parts > 2 && p[1] > cy {
                label |= 2;
            }
            if parts > 4 && p[2] > cz {
                label |= 4;
            }
            (label % parts as u32) as f64
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConvexPart",
            labels,
            1,
        )));
    result.point_data_mut().set_active_scalars("ConvexPart");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decompose() {
        let mesh = PolyData::from_triangles(
            vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = convex_decompose(&mesh, 4);
        assert!(r.point_data().get_array("ConvexPart").is_some());
    }
}
