//! Build topology dual: vertices become faces, faces become vertices.
use crate::data::{CellArray, Points, PolyData};
pub fn topology_dual(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let nv = mesh.points.len();
    // Face centroids become new vertices
    let mut pts = Points::<f64>::new();
    for c in &cells {
        if c.is_empty() {
            pts.push([0.0, 0.0, 0.0]);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &v in c {
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = c.len() as f64;
        pts.push([cx / n, cy / n, cz / n]);
    }
    // For each original vertex, find adjacent faces -> new face
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for (ci, c) in cells.iter().enumerate() {
        for &v in c {
            vf[v as usize].push(ci);
        }
    }
    // Build adjacency to order faces around each vertex
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let a = c[i] as usize;
            let b = c[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut polys = CellArray::new();
    for vi in 0..nv {
        if vf[vi].len() < 3 {
            continue;
        }
        // Sort faces around vertex by angle
        let p = mesh.points.get(vi);
        let mut faces_angle: Vec<(usize, f64)> = vf[vi]
            .iter()
            .map(|&fi| {
                let cp = pts.get(fi);
                let dx = cp[0] - p[0];
                let dy = cp[1] - p[1];
                (fi, dy.atan2(dx))
            })
            .collect();
        faces_angle.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let ids: Vec<i64> = faces_angle.iter().map(|&(fi, _)| fi as i64).collect();
        if ids.len() >= 3 {
            polys.push_cell(&ids);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let d = topology_dual(&m);
        assert_eq!(d.points.len(), 3);
        assert!(d.polys.num_cells() >= 1);
    }
}
