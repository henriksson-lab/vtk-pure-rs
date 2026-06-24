//! Compute histogram of edge lengths in a mesh.
use crate::data::PolyData;

pub struct EdgeHistogram {
    pub bins: Vec<usize>,
    pub bin_edges: Vec<f64>,
    pub min_length: f64,
    pub max_length: f64,
    pub mean_length: f64,
    pub total_edges: usize,
}

pub fn edge_histogram(mesh: &PolyData, n_bins: usize) -> EdgeHistogram {
    let n = mesh.points.len();
    let nb = n_bins.max(1);
    let mut lengths = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= n || b >= n {
                continue;
            }
            let e = if a < b { (a, b) } else { (b, a) };
            if !seen.insert(e) {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            lengths.push(
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                    .sqrt(),
            );
        }
    }
    if lengths.is_empty() {
        return EdgeHistogram {
            bins: vec![0; nb],
            bin_edges: vec![0.0; nb + 1],
            min_length: 0.0,
            max_length: 0.0,
            mean_length: 0.0,
            total_edges: 0,
        };
    }
    let min_l = lengths.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_l = lengths.iter().cloned().fold(0.0f64, f64::max);
    let mean_l = lengths.iter().sum::<f64>() / lengths.len() as f64;
    let range = (max_l - min_l).max(1e-15);
    let mut bins = vec![0usize; nb];
    let bin_edges: Vec<f64> = (0..=nb)
        .map(|i| min_l + range * i as f64 / nb as f64)
        .collect();
    for &l in &lengths {
        let idx = ((l - min_l) / range * nb as f64).floor() as usize;
        bins[idx.min(nb - 1)] += 1;
    }
    EdgeHistogram {
        bins,
        bin_edges,
        min_length: min_l,
        max_length: max_l,
        mean_length: mean_l,
        total_edges: lengths.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_histogram() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let h = edge_histogram(&mesh, 5);
        assert_eq!(h.total_edges, 3);
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
    }
}
