//! Laplacian smoothing that allows boundary vertices to slide along boundary edges.
use crate::data::PolyData;
pub fn smooth_boundary_slide(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
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
    let mut boundary_edges: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary_edges.entry(a).or_default().push(b);
            boundary_edges.entry(b).or_default().push(a);
        }
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            if let Some(bnb) = boundary_edges.get(&i) {
                // Boundary: average only boundary neighbors
                if bnb.len() != 2 {
                    continue;
                }
                let avg = [
                    (pos[bnb[0]][0] + pos[bnb[1]][0]) / 2.0,
                    (pos[bnb[0]][1] + pos[bnb[1]][1]) / 2.0,
                    (pos[bnb[0]][2] + pos[bnb[1]][2]) / 2.0,
                ];
                new_pos[i][0] = pos[i][0] + lambda * (avg[0] - pos[i][0]);
                new_pos[i][1] = pos[i][1] + lambda * (avg[1] - pos[i][1]);
                new_pos[i][2] = pos[i][2] + lambda * (avg[2] - pos[i][2]);
            } else {
                let k = nb[i].len() as f64;
                let mut avg = [0.0, 0.0, 0.0];
                for &j in &nb[i] {
                    avg[0] += pos[j][0];
                    avg[1] += pos[j][1];
                    avg[2] += pos[j][2];
                }
                new_pos[i][0] = pos[i][0] + lambda * (avg[0] / k - pos[i][0]);
                new_pos[i][1] = pos[i][1] + lambda * (avg[1] / k - pos[i][1]);
                new_pos[i][2] = pos[i][2] + lambda * (avg[2] / k - pos[i][2]);
            }
        }
        pos = new_pos;
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
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = smooth_boundary_slide(&m, 5, 0.3);
        assert_eq!(r.points.len(), 4);
    }
}
