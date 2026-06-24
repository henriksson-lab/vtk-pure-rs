//! Find closest mesh vertex for each query point.
use crate::data::PolyData;
pub fn closest_vertices(mesh: &PolyData, query_points: &[[f64; 3]]) -> Vec<(usize, f64)> {
    let n = mesh.points.len();
    query_points
        .iter()
        .map(|q| {
            let mut best = (0, f64::INFINITY);
            for i in 0..n {
                let p = mesh.points.get(i);
                let d = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2);
                if d < best.1 {
                    best = (i, d);
                }
            }
            (best.0, best.1.sqrt())
        })
        .collect()
}
pub fn closest_vertex_ids(mesh: &PolyData, query_points: &[[f64; 3]]) -> Vec<usize> {
    closest_vertices(mesh, query_points)
        .iter()
        .map(|&(i, _)| i)
        .collect()
}
pub fn closest_vertex_distances(mesh: &PolyData, query_points: &[[f64; 3]]) -> Vec<f64> {
    closest_vertices(mesh, query_points)
        .iter()
        .map(|&(_, d)| d)
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = closest_vertices(&m, &[[0.1, 0.1, 0.0], [0.9, 0.1, 0.0]]);
        assert_eq!(r[0].0, 0);
        assert_eq!(r[1].0, 1);
    }
    #[test]
    fn test_ids() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [2.5, 5.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let ids = closest_vertex_ids(&m, &[[0.0, 0.0, 0.0]]);
        assert_eq!(ids[0], 0);
    }
}
