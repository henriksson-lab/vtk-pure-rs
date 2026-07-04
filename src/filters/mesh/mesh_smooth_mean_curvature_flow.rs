//! Mean curvature flow smoothing.
use crate::data::{Points, PolyData};

pub fn mean_curvature_flow(mesh: &PolyData, steps: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_neighbor_pair(&mut nb, n, cell[i], cell[(i + 1) % nc]);
        }
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..steps {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                lap[0] += pos[j][0] - pos[i][0];
                lap[1] += pos[j][1] - pos[i][1];
                lap[2] += pos[j][2] - pos[i][2];
            }
            let k = nb[i].len() as f64;
            new_pos[i][0] += dt * lap[0] / k;
            new_pos[i][1] += dt * lap[1] / k;
            new_pos[i][2] += dt * lap[2] / k;
        }
        pos = new_pos;
    }
    let mut r = mesh.clone();
    r.points = Points::from(pos);
    r
}

fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
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
        let r = mean_curvature_flow(&m, 5, 0.1);
        assert_eq!(r.points.len(), 4);
    }
}
