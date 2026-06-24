//! Estimate mesh feature size (average edge length, minimum edge, etc).
use crate::data::PolyData;
pub struct FeatureSize {
    pub avg_edge: f64,
    pub min_edge: f64,
    pub max_edge: f64,
    pub median_edge: f64,
}
pub fn estimate_feature_size(mesh: &PolyData) -> FeatureSize {
    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
    }
    if edges.is_empty() {
        return FeatureSize {
            avg_edge: 0.0,
            min_edge: 0.0,
            max_edge: 0.0,
            median_edge: 0.0,
        };
    }
    let mut lengths: Vec<f64> = edges
        .iter()
        .map(|&(a, b)| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
        })
        .collect();
    lengths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = lengths.len();
    FeatureSize {
        avg_edge: lengths.iter().sum::<f64>() / n as f64,
        min_edge: lengths[0],
        max_edge: lengths[n - 1],
        median_edge: if n % 2 == 0 {
            (lengths[n / 2 - 1] + lengths[n / 2]) / 2.0
        } else {
            lengths[n / 2]
        },
    }
}
pub fn is_uniform_mesh(mesh: &PolyData, tolerance: f64) -> bool {
    let fs = estimate_feature_size(mesh);
    if fs.avg_edge < 1e-15 {
        return true;
    }
    (fs.max_edge - fs.min_edge) / fs.avg_edge < tolerance
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
        let fs = estimate_feature_size(&m);
        assert!(fs.avg_edge > 0.5);
        assert!(fs.min_edge > 0.0);
    }
    #[test]
    fn test_uniform() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.866, 0.0]],
            vec![[0, 1, 2]],
        ); // near equilateral
        let u = is_uniform_mesh(&m, 0.5);
        assert!(u);
    }
}
