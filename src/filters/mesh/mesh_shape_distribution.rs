//! Shape distribution descriptors (D2, A3, etc.) for shape retrieval.
use crate::data::PolyData;
pub struct ShapeDistribution {
    pub histogram: Vec<f64>,
    pub min: f64,
    pub max: f64,
}
/// D2: distance between random point pairs.
pub fn d2_distribution(
    mesh: &PolyData,
    num_samples: usize,
    num_bins: usize,
    seed: u64,
) -> ShapeDistribution {
    let n = mesh.points.len();
    if n < 2 {
        return ShapeDistribution {
            histogram: vec![],
            min: 0.0,
            max: 0.0,
        };
    }
    let ns = num_samples.max(10);
    let nb = num_bins.max(2);
    let mut rng = seed;
    let mut dists = Vec::with_capacity(ns);
    for _ in 0..ns {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let i = (rng >> 33) as usize % n;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % n;
        if i == j {
            continue;
        }
        let p = mesh.points.get(i);
        let q = mesh.points.get(j);
        dists.push(((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt());
    }
    build_hist(&dists, nb)
}
/// A3: angle between random triples.
pub fn a3_distribution(
    mesh: &PolyData,
    num_samples: usize,
    num_bins: usize,
    seed: u64,
) -> ShapeDistribution {
    let n = mesh.points.len();
    if n < 3 {
        return ShapeDistribution {
            histogram: vec![],
            min: 0.0,
            max: 0.0,
        };
    }
    let ns = num_samples.max(10);
    let nb = num_bins.max(2);
    let mut rng = seed;
    let mut angles = Vec::with_capacity(ns);
    for _ in 0..ns {
        let mut pick = || -> usize {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (rng >> 33) as usize % n
        };
        let i = pick();
        let j = pick();
        let k = pick();
        if i == j || j == k || i == k {
            continue;
        }
        let p = mesh.points.get(i);
        let a = mesh.points.get(j);
        let b = mesh.points.get(k);
        let v1 = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
        let v2 = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
        let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
        let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
        if l1 > 1e-15 && l2 > 1e-15 {
            angles.push(
                ((v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]) / (l1 * l2))
                    .clamp(-1.0, 1.0)
                    .acos(),
            );
        }
    }
    build_hist(&angles, nb)
}
/// D1: distance from centroid.
pub fn d1_distribution(mesh: &PolyData, num_bins: usize) -> ShapeDistribution {
    let n = mesh.points.len();
    if n == 0 {
        return ShapeDistribution {
            histogram: vec![],
            min: 0.0,
            max: 0.0,
        };
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    build_hist(&dists, num_bins)
}
pub fn distribution_distance(a: &ShapeDistribution, b: &ShapeDistribution) -> f64 {
    let n = a.histogram.len().min(b.histogram.len());
    if n == 0 {
        return f64::INFINITY;
    }
    (0..n)
        .map(|i| (a.histogram[i] - b.histogram[i]).powi(2))
        .sum::<f64>()
        .sqrt()
}
fn build_hist(vals: &[f64], num_bins: usize) -> ShapeDistribution {
    if vals.is_empty() {
        return ShapeDistribution {
            histogram: vec![],
            min: 0.0,
            max: 0.0,
        };
    }
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let nb = num_bins.max(1);
    let mut hist = vec![0.0f64; nb];
    for &v in vals {
        let bi = (((v - mn) / range * nb as f64).floor() as usize).min(nb - 1);
        hist[bi] += 1.0;
    }
    let total: f64 = hist.iter().sum();
    if total > 0.0 {
        for h in &mut hist {
            *h /= total;
        }
    }
    ShapeDistribution {
        histogram: hist,
        min: mn,
        max: mx,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_d2() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let d = d2_distribution(&m, 100, 10, 42);
        assert_eq!(d.histogram.len(), 10);
    }
    #[test]
    fn test_a3() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let a = a3_distribution(&m, 100, 10, 42);
        assert_eq!(a.histogram.len(), 10);
    }
    #[test]
    fn test_d1() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = d1_distribution(&m, 5);
        assert_eq!(d.histogram.len(), 5);
    }
    #[test]
    fn test_dist() {
        let a = ShapeDistribution {
            histogram: vec![0.5, 0.5],
            min: 0.0,
            max: 1.0,
        };
        let b = ShapeDistribution {
            histogram: vec![0.3, 0.7],
            min: 0.0,
            max: 1.0,
        };
        let d = distribution_distance(&a, &b);
        assert!(d > 0.0);
    }
}
