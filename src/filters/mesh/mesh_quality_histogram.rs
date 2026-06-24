//! Compute quality histogram for all triangle faces.
use crate::data::PolyData;

pub struct QualityHistogram {
    pub bins: Vec<usize>,
    pub min_quality: f64,
    pub max_quality: f64,
    pub mean_quality: f64,
    pub n_degenerate: usize,
}

pub fn quality_histogram(mesh: &PolyData, n_bins: usize) -> QualityHistogram {
    let n = mesh.points.len();
    let nb = n_bins.max(1);
    let mut qualities = Vec::new();
    let mut n_degen = 0usize;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            n_degen += 1;
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            n_degen += 1;
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let ab =
            ((pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2) + (pb[2] - pa[2]).powi(2)).sqrt();
        let bc =
            ((pc[0] - pb[0]).powi(2) + (pc[1] - pb[1]).powi(2) + (pc[2] - pb[2]).powi(2)).sqrt();
        let ca =
            ((pa[0] - pc[0]).powi(2) + (pa[1] - pc[1]).powi(2) + (pa[2] - pc[2]).powi(2)).sqrt();
        let s = (ab + bc + ca) / 2.0;
        let area = (s * (s - ab).max(0.0) * (s - bc).max(0.0) * (s - ca).max(0.0)).sqrt();
        let longest = ab.max(bc).max(ca);
        let ideal = longest * longest * 3.0f64.sqrt() / 4.0;
        let q = if ideal > 1e-15 { area / ideal } else { 0.0 };
        qualities.push(q.clamp(0.0, 1.0));
    }
    if qualities.is_empty() {
        return QualityHistogram {
            bins: vec![0; nb],
            min_quality: 0.0,
            max_quality: 0.0,
            mean_quality: 0.0,
            n_degenerate: n_degen,
        };
    }
    let min_q = qualities.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_q = qualities.iter().cloned().fold(0.0f64, f64::max);
    let mean_q = qualities.iter().sum::<f64>() / qualities.len() as f64;
    let mut bins = vec![0usize; nb];
    for &q in &qualities {
        let idx = (q * nb as f64).floor() as usize;
        bins[idx.min(nb - 1)] += 1;
    }
    QualityHistogram {
        bins,
        min_quality: min_q,
        max_quality: max_q,
        mean_quality: mean_q,
        n_degenerate: n_degen,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quality_hist() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let h = quality_histogram(&mesh, 5);
        assert_eq!(h.bins.iter().sum::<usize>(), 2);
        assert!(h.mean_quality > 0.5);
    }
}
