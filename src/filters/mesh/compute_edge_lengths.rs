//! Compute per-edge length statistics.

use crate::data::PolyData;

/// Edge length statistics for a mesh.
pub struct EdgeLengthStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub total_edges: usize,
}

/// Compute edge length statistics.
pub fn edge_length_stats(mesh: &PolyData) -> EdgeLengthStats {
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
        return EdgeLengthStats {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            std_dev: 0.0,
            total_edges: 0,
        };
    }
    let lengths: Vec<f64> = edges
        .iter()
        .map(|&(a, b)| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
        })
        .collect();
    let n = lengths.len() as f64;
    let mn = lengths.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = lengths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = lengths.iter().sum::<f64>() / n;
    let variance = lengths.iter().map(|&l| (l - mean).powi(2)).sum::<f64>() / n;
    EdgeLengthStats {
        min: mn,
        max: mx,
        mean,
        std_dev: variance.sqrt(),
        total_edges: lengths.len(),
    }
}

/// Get shortest edge.
pub fn shortest_edge(mesh: &PolyData) -> (usize, usize, f64) {
    let mut best = (0, 0, f64::INFINITY);
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let key = (a.min(b), a.max(b));
            if seen.insert(key) {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                if d < best.2 {
                    best = (a, b, d);
                }
            }
        }
    }
    best
}

/// Get longest edge.
pub fn longest_edge(mesh: &PolyData) -> (usize, usize, f64) {
    let mut best = (0, 0, 0.0f64);
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let key = (a.min(b), a.max(b));
            if seen.insert(key) {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                if d > best.2 {
                    best = (a, b, d);
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stats() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let stats = edge_length_stats(&mesh);
        assert_eq!(stats.total_edges, 3);
        assert!(stats.min > 0.0);
        assert!(stats.max >= stats.min);
    }
    #[test]
    fn test_shortest_longest() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (_, _, short) = shortest_edge(&mesh);
        let (_, _, long) = longest_edge(&mesh);
        assert!((short - 1.0).abs() < 1e-10);
        assert!((long - (9.0f64 + 1.0).sqrt()).abs() < 1e-10);
    }
}
