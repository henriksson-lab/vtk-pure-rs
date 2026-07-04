//! Check how many edges satisfy the Delaunay condition (empty circumcircle).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn delaunay_check(mesh: &PolyData) -> (f64, PolyData) {
    let n = mesh.points.len();
    let mut tris = Vec::new();
    let mut tri_cell_ids = Vec::new();
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() == 3 {
            let Some(a) = valid_point_id(cell[0], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[1], n) else {
                continue;
            };
            let Some(c) = valid_point_id(cell[2], n) else {
                continue;
            };
            tris.push([a, b, c]);
            tri_cell_ids.push(ci);
        }
    }
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
    let mut edge_ok = vec![1.0f64; mesh.polys.num_cells()];
    for (_, faces) in &edge_tris {
        if faces.len() != 2 {
            continue;
        }
        total += 1;
        let t0 = faces[0];
        let t1 = faces[1];
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
                edge_ok[tri_cell_ids[t0]] = 0.0;
                edge_ok[tri_cell_ids[t1]] = 0.0;
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

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < n)
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
