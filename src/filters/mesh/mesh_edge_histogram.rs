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
    for cell in mesh.lines.iter() {
        for pair in cell.windows(2) {
            insert_edge_length(mesh, n, pair[0], pair[1], &mut seen, &mut lengths);
        }
    }
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge_length(
                mesh,
                n,
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut seen,
                &mut lengths,
            );
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge_length(mesh, n, tri[0], tri[1], &mut seen, &mut lengths);
            insert_edge_length(mesh, n, tri[1], tri[2], &mut seen, &mut lengths);
            insert_edge_length(mesh, n, tri[2], tri[0], &mut seen, &mut lengths);
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

fn insert_edge_length(
    mesh: &PolyData,
    n: usize,
    a: i64,
    b: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    lengths: &mut Vec<f64>,
) {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a >= n || b >= n || a == b {
        return;
    }
    let e = if a < b { (a, b) } else { (b, a) };
    if !seen.insert(e) {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    lengths
        .push(((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt());
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
