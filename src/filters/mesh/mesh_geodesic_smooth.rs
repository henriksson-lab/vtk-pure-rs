//! Geodesic-distance-weighted smoothing.
use crate::data::PolyData;
pub fn geodesic_weighted_smooth(mesh: &PolyData, iterations: usize, sigma: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || sigma <= 0.0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut nb);
        }
    }
    let s2 = 2.0 * sigma * sigma;
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let mut sum = [0.0, 0.0, 0.0];
            let mut wsum = 0.0;
            for &(j, d) in &nb[i] {
                let w = (-d * d / s2).exp();
                sum[0] += w * prev[j][0];
                sum[1] += w * prev[j][1];
                sum[2] += w * prev[j][2];
                wsum += w;
            }
            if wsum > 1e-15 {
                pos[i] = [sum[0] / wsum, sum[1] / wsum, sum[2] / wsum];
            }
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}

fn add_edge(mesh: &PolyData, a: i64, b: i64, nb: &mut [Vec<(usize, f64)>]) {
    let n = nb.len();
    let Some(a) = valid_point_id(a, n) else {
        return;
    };
    let Some(b) = valid_point_id(b, n) else {
        return;
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !nb[a].iter().any(|&(x, _)| x == b) {
        nb[a].push((b, d));
    }
    if !nb[b].iter().any(|&(x, _)| x == a) {
        nb[b].push((a, d));
    }
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
        let r = geodesic_weighted_smooth(&m, 3, 1.0);
        assert_eq!(r.points.len(), 4);
    }
}
