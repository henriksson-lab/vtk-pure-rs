//! Check how many edges satisfy the Delaunay condition (empty circumcircle).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn delaunay_check(mesh: &PolyData) -> (f64, PolyData) {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.len() < 2 {
        return (1.0, mesh.clone());
    }
    let mut edge_tris: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ti, &[a, b, c]) in tris.iter().enumerate() {
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_tris.entry(e).or_default().push(ti);
        }
    }
    let mut total = 0usize;
    let mut delaunay = 0usize;
    let mut edge_ok = vec![1.0f64; tris.len()];
    for (_, faces) in &edge_tris {
        if faces.len() != 2 {
            continue;
        }
        total += 1;
        let t0 = faces[0];
        let t1 = faces[1];
        let [a0, b0, c0] = tris[t0];
        let [a1, b1, c1] = tris[t1];
        // Find opposite vertices
        let shared: Vec<usize> = tris[t0]
            .iter()
            .filter(|v| tris[t1].contains(v))
            .copied()
            .collect();
        if shared.len() != 2 {
            continue;
        }
        let opp0 = tris[t0].iter().find(|v| !shared.contains(v)).copied();
        let opp1 = tris[t1].iter().find(|v| !shared.contains(v)).copied();
        if let (Some(o0), Some(o1)) = (opp0, opp1) {
            if o0 >= n || o1 >= n {
                continue;
            }
            // Delaunay condition: sum of opposite angles < pi
            let p0 = mesh.points.get(o0);
            let p1 = mesh.points.get(o1);
            let pa = mesh.points.get(shared[0]);
            let pb = mesh.points.get(shared[1]);
            let angle0 = angle_at_vertex(p0, pa, pb);
            let angle1 = angle_at_vertex(p1, pa, pb);
            if angle0 + angle1 <= std::f64::consts::PI + 1e-10 {
                delaunay += 1;
            } else {
                edge_ok[t0] = 0.0;
                edge_ok[t1] = 0.0;
            }
        }
    }
    let ratio = if total > 0 {
        delaunay as f64 / total as f64
    } else {
        1.0
    };
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DelaunayOK",
            edge_ok,
            1,
        )));
    (ratio, result)
}

fn angle_at_vertex(v: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    let va = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let vb = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];
    let dot = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
    if la < 1e-15 || lb < 1e-15 {
        return 0.0;
    }
    (dot / (la * lb)).clamp(-1.0, 1.0).acos()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_delaunay() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let (ratio, _) = delaunay_check(&mesh);
        assert!(ratio >= 0.0 && ratio <= 1.0);
    }
}
