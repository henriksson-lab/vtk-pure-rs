//! Optimal Delaunay Triangulation (ODT) mesh improvement.
use crate::data::PolyData;
pub fn odt_smooth(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    let mut boundary: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    // ODT: move each interior vertex to circumcenter-weighted average
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if boundary.contains(&i) || nb[i].is_empty() {
                continue;
            }
            // Use area-weighted centroid of adjacent triangles as target
            let mut target = [0.0, 0.0, 0.0];
            let mut total_area = 0.0;
            for cell in mesh.polys.iter() {
                if cell.len() != 3 {
                    continue;
                }
                if !cell.iter().any(|&v| v as usize == i) {
                    continue;
                }
                let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
                let p = [prev[ids[0]], prev[ids[1]], prev[ids[2]]];
                let cx = (p[0][0] + p[1][0] + p[2][0]) / 3.0;
                let cy = (p[0][1] + p[1][1] + p[2][1]) / 3.0;
                let cz = (p[0][2] + p[1][2] + p[2][2]) / 3.0;
                let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
                let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
                let area = 0.5
                    * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                    .sqrt();
                target[0] += cx * area;
                target[1] += cy * area;
                target[2] += cz * area;
                total_area += area;
            }
            if total_area > 1e-15 {
                pos[i] = [
                    target[0] / total_area,
                    target[1] / total_area,
                    target[2] / total_area,
                ];
            }
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
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
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [1.0, 0.5, 0.0],
            ],
            vec![[0, 1, 4], [1, 3, 4], [3, 2, 4], [2, 0, 4]],
        );
        let r = odt_smooth(&m, 10);
        assert_eq!(r.points.len(), 5);
    }
}
