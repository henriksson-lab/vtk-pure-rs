//! Detect self-intersecting triangles in a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn detect_self_intersections(mesh: &PolyData) -> PolyData {
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3], Vec<usize>, usize)> = mesh
        .polys
        .iter()
        .enumerate()
        .filter(|(_, c)| valid_cell(mesh, c))
        .flat_map(|(cell_id, c)| {
            let vids: Vec<usize> = c.iter().map(|&id| id as usize).collect();
            let a = mesh.points.get(c[0] as usize);
            (1..c.len() - 1).map(move |i| {
                (
                    a,
                    mesh.points.get(c[i] as usize),
                    mesh.points.get(c[i + 1] as usize),
                    vids.clone(),
                    cell_id,
                )
            })
        })
        .collect();
    let nt = tris.len();
    let mut intersecting = vec![0.0f64; mesh.polys.num_cells()];
    // Check each pair of non-adjacent triangles
    for i in 0..nt {
        for j in (i + 1)..nt {
            // Skip if they share a vertex
            let shared = tris[i].3.iter().any(|v| tris[j].3.contains(v));
            if shared {
                continue;
            }
            let bbox_i = tri_bbox(&tris[i].0, &tris[i].1, &tris[i].2);
            let bbox_j = tri_bbox(&tris[j].0, &tris[j].1, &tris[j].2);
            if !aabb_overlap(&bbox_i, &bbox_j) {
                continue;
            }
            if tri_tri_intersect(
                &tris[i].0, &tris[i].1, &tris[i].2, &tris[j].0, &tris[j].1, &tris[j].2,
            ) {
                intersecting[tris[i].4] = 1.0;
                intersecting[tris[j].4] = 1.0;
            }
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

fn valid_cell(mesh: &PolyData, cell: &[i64]) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&point_id| point_id >= 0 && (point_id as usize) < mesh.points.len())
}

fn tri_bbox(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> ([f64; 3], [f64; 3]) {
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for p in [a, b, c] {
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

fn tri_tri_intersect(
    a0: &[f64; 3],
    a1: &[f64; 3],
    a2: &[f64; 3],
    b0: &[f64; 3],
    b1: &[f64; 3],
    b2: &[f64; 3],
) -> bool {
    let edges_a = [(a0, a1), (a1, a2), (a2, a0)];
    let edges_b = [(b0, b1), (b1, b2), (b2, b0)];

    for &(p, q) in &edges_a {
        if edge_tri_intersect(p, q, b0, b1, b2) {
            return true;
        }
    }
    for &(p, q) in &edges_b {
        if edge_tri_intersect(p, q, a0, a1, a2) {
            return true;
        }
    }
    false
}

fn edge_tri_intersect(
    o: &[f64; 3],
    end: &[f64; 3],
    v0: &[f64; 3],
    v1: &[f64; 3],
    v2: &[f64; 3],
) -> bool {
    let d = [end[0] - o[0], end[1] - o[1], end[2] - o[2]];
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return false;
    }
    let f = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-6 && t < 1.0 - 1e-6
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
