//! Compute histogram of vertex-to-vertex distances.
use crate::data::PolyData;
pub struct DistanceHistogram {
    pub bins: Vec<usize>,
    pub min_dist: f64,
    pub max_dist: f64,
    pub bin_width: f64,
    pub mean_dist: f64,
}
pub fn edge_distance_histogram(mesh: &PolyData, num_bins: usize) -> DistanceHistogram {
    let nb = num_bins.max(1);
    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.lines.iter() {
        for pair in cell.windows(2) {
            insert_valid_edge(&mut edges, pair[0], pair[1], mesh.points.len());
        }
    }
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            insert_valid_edge(&mut edges, cell[i], cell[(i + 1) % nc], mesh.points.len());
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_valid_edge(&mut edges, tri[0], tri[1], mesh.points.len());
            insert_valid_edge(&mut edges, tri[1], tri[2], mesh.points.len());
            insert_valid_edge(&mut edges, tri[2], tri[0], mesh.points.len());
        }
    }
    let dists: Vec<f64> = edges
        .iter()
        .map(|&(a, b)| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
        })
        .collect();
    if dists.is_empty() {
        return DistanceHistogram {
            bins: vec![0; nb],
            min_dist: 0.0,
            max_dist: 0.0,
            bin_width: 0.0,
            mean_dist: 0.0,
        };
    }
    let mn = dists.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = dists.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bw = (mx - mn).max(1e-15) / nb as f64;
    let mut bins = vec![0usize; nb];
    for &d in &dists {
        let bi = (((d - mn) / bw).floor() as usize).min(nb - 1);
        bins[bi] += 1;
    }
    let mean = dists.iter().sum::<f64>() / dists.len() as f64;
    DistanceHistogram {
        bins,
        min_dist: mn,
        max_dist: mx,
        bin_width: bw,
        mean_dist: mean,
    }
}
pub fn all_pairs_distance_histogram(
    mesh: &PolyData,
    num_bins: usize,
    max_pairs: usize,
) -> DistanceHistogram {
    let nb = num_bins.max(1);
    let n = mesh.points.len();
    let mut dists = Vec::new();
    let limit = max_pairs.min(n.saturating_mul(n.saturating_sub(1)) / 2);
    let mut count = 0;
    'outer: for i in 0..n {
        for j in i + 1..n {
            if count >= limit {
                break 'outer;
            }
            let pa = mesh.points.get(i);
            let pb = mesh.points.get(j);
            dists.push(
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                    .sqrt(),
            );
            count += 1;
        }
    }
    if dists.is_empty() {
        return DistanceHistogram {
            bins: vec![0; nb],
            min_dist: 0.0,
            max_dist: 0.0,
            bin_width: 0.0,
            mean_dist: 0.0,
        };
    }
    let mn = dists.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = dists.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bw = (mx - mn).max(1e-15) / nb as f64;
    let mut bins = vec![0usize; nb];
    for &d in &dists {
        let bi = (((d - mn) / bw).floor() as usize).min(nb - 1);
        bins[bi] += 1;
    }
    DistanceHistogram {
        bins,
        min_dist: mn,
        max_dist: mx,
        bin_width: bw,
        mean_dist: dists.iter().sum::<f64>() / dists.len() as f64,
    }
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

fn insert_valid_edge(
    edges: &mut std::collections::HashSet<(usize, usize)>,
    a: i64,
    b: i64,
    num_points: usize,
) {
    let Some(a) = valid_point_id(a, num_points) else {
        return;
    };
    let Some(b) = valid_point_id(b, num_points) else {
        return;
    };
    if a != b {
        edges.insert((a.min(b), a.max(b)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edge() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let h = edge_distance_histogram(&m, 5);
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
    }
    #[test]
    fn test_pairs() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let h = all_pairs_distance_histogram(&m, 5, 100);
        assert_eq!(h.bins.iter().sum::<usize>(), 3);
    }

    #[test]
    fn test_edge_includes_polydata_lines_and_strips() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([2.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.lines.push_cell(&[0, 1, 2]);
        m.strips.push_cell(&[0, 1, 3]);

        let h = edge_distance_histogram(&m, 4);
        assert_eq!(h.bins.iter().sum::<usize>(), 4);
    }

    #[test]
    fn test_empty_histogram_keeps_requested_bins() {
        let m = PolyData::new();
        let h = edge_distance_histogram(&m, 4);
        assert_eq!(h.bins, vec![0; 4]);
        let h = all_pairs_distance_histogram(&m, 4, 100);
        assert_eq!(h.bins, vec![0; 4]);
    }
}
