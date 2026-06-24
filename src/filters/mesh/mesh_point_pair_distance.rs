//! Compute pairwise distances between vertices.
use crate::data::PolyData;
pub fn diameter(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    let mut max_d = 0.0f64;
    for i in 0..n {
        let pi = mesh.points.get(i);
        for j in i + 1..n {
            let pj = mesh.points.get(j);
            let d = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            max_d = max_d.max(d);
        }
    }
    max_d.sqrt()
}
pub fn farthest_pair(mesh: &PolyData) -> (usize, usize, f64) {
    let n = mesh.points.len();
    let mut best = (0, 0, 0.0f64);
    for i in 0..n {
        let pi = mesh.points.get(i);
        for j in i + 1..n {
            let pj = mesh.points.get(j);
            let d = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            if d > best.2 {
                best = (i, j, d);
            }
        }
    }
    (best.0, best.1, best.2.sqrt())
}
pub fn closest_pair(mesh: &PolyData) -> (usize, usize, f64) {
    let n = mesh.points.len();
    if n < 2 {
        return (0, 0, 0.0);
    }
    let mut best = (0, 1, f64::INFINITY);
    for i in 0..n {
        let pi = mesh.points.get(i);
        for j in i + 1..n {
            let pj = mesh.points.get(j);
            let d = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            if d < best.2 {
                best = (i, j, d);
            }
        }
    }
    (best.0, best.1, best.2.sqrt())
}
pub fn average_nearest_neighbor_distance(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..n {
        let pi = mesh.points.get(i);
        let mut best = f64::INFINITY;
        for j in 0..n {
            if i == j {
                continue;
            }
            let pj = mesh.points.get(j);
            let d = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            best = best.min(d);
        }
        total += best.sqrt();
    }
    total / n as f64
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_diameter() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((diameter(&m) - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_pairs() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (a, b, d) = farthest_pair(&m);
        assert!(d > 0.9);
        let (a2, b2, d2) = closest_pair(&m);
        assert!(d2 > 0.0 && d2 <= 1.01);
    }
    #[test]
    fn test_ann() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = average_nearest_neighbor_distance(&m);
        assert!(d > 0.0);
    }
}
