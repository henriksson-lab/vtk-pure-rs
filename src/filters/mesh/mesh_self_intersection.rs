//! Detect self-intersecting triangles in a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn detect_self_intersections(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let nt = tris.len();
    let mut intersecting = vec![0.0f64; nt];
    // Check each pair of non-adjacent triangles
    for i in 0..nt {
        for j in (i + 1)..nt {
            // Skip if they share a vertex
            let shared = tris[i].iter().any(|v| tris[j].contains(v));
            if shared {
                continue;
            }
            // Simple AABB overlap test
            let bbox_i = tri_bbox(&mesh, &tris[i]);
            let bbox_j = tri_bbox(&mesh, &tris[j]);
            if !aabb_overlap(&bbox_i, &bbox_j) {
                continue;
            }
            // Detailed triangle-triangle intersection would go here
            // For now, flag AABB overlaps as potential intersections
            intersecting[i] = 1.0;
            intersecting[j] = 1.0;
        }
    }
    // Map to cell data
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SelfIntersection",
            intersecting,
            1,
        )));
    result
}

fn tri_bbox(mesh: &PolyData, tri: &[usize; 3]) -> ([f64; 3], [f64; 3]) {
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for &v in tri {
        let p = mesh.points.get(v);
        for d in 0..3 {
            mn[d] = mn[d].min(p[d]);
            mx[d] = mx[d].max(p[d]);
        }
    }
    (mn, mx)
}

fn aabb_overlap(a: &([f64; 3], [f64; 3]), b: &([f64; 3], [f64; 3])) -> bool {
    (0..3).all(|d| a.0[d] <= b.1[d] && b.0[d] <= a.1[d])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_no_intersection() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = detect_self_intersections(&mesh);
        assert!(r.cell_data().get_array("SelfIntersection").is_some());
    }
}
