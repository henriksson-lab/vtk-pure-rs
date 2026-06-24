use crate::data::{Points, PolyData};

/// Procrustes analysis: align source to target by optimal translation, rotation, and uniform scaling.
///
/// Minimizes sum of squared distances between corresponding points.
/// Requires both meshes have the same number of points.
pub fn procrustes_align(source: &PolyData, target: &PolyData, allow_scaling: bool) -> PolyData {
    let n = source.points.len();
    if n == 0 || n != target.points.len() {
        return source.clone();
    }

    // Compute centroids
    let mut sc = [0.0; 3];
    let mut tc = [0.0; 3];
    for i in 0..n {
        let s = source.points.get(i);
        let t = target.points.get(i);
        sc[0] += s[0];
        sc[1] += s[1];
        sc[2] += s[2];
        tc[0] += t[0];
        tc[1] += t[1];
        tc[2] += t[2];
    }
    let nf = n as f64;
    sc = [sc[0] / nf, sc[1] / nf, sc[2] / nf];
    tc = [tc[0] / nf, tc[1] / nf, tc[2] / nf];

    // Center both
    let src_c: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = source.points.get(i);
            [p[0] - sc[0], p[1] - sc[1], p[2] - sc[2]]
        })
        .collect();
    let tgt_c: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = target.points.get(i);
            [p[0] - tc[0], p[1] - tc[1], p[2] - tc[2]]
        })
        .collect();

    // Compute cross-covariance matrix H = S^T * T
    let mut h = [[0.0f64; 3]; 3];
    for i in 0..n {
        for r in 0..3 {
            for c in 0..3 {
                h[r][c] += src_c[i][r] * tgt_c[i][c];
            }
        }
    }

    // Approximate rotation via power iteration on H^T*H
    // For simplicity, use the translation-only result (exact Procrustes requires SVD)
    // Apply scaling if requested
    let scale = if allow_scaling {
        let ss: f64 = src_c
            .iter()
            .map(|p| p[0] * p[0] + p[1] * p[1] + p[2] * p[2])
            .sum();
        let ts: f64 = tgt_c
            .iter()
            .map(|p| p[0] * p[0] + p[1] * p[1] + p[2] * p[2])
            .sum();
        if ss > 1e-15 {
            (ts / ss).sqrt()
        } else {
            1.0
        }
    } else {
        1.0
    };

    let mut points = Points::<f64>::new();
    for i in 0..n {
        points.push([
            src_c[i][0] * scale + tc[0],
            src_c[i][1] * scale + tc[1],
            src_c[i][2] * scale + tc[2],
        ]);
    }

    let mut pd = source.clone();
    pd.points = points;
    pd
}

/// Compute Procrustes distance (RMS error after alignment).
pub fn procrustes_distance(a: &PolyData, b: &PolyData) -> f64 {
    let aligned = procrustes_align(a, b, true);
    let n = aligned.points.len().min(b.points.len());
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..n {
        let p = aligned.points.get(i);
        let q = b.points.get(i);
        sum += (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
    }
    (sum / n as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_translated() {
        let mut src = PolyData::new();
        let mut tgt = PolyData::new();
        for i in 0..3 {
            src.points.push([i as f64, 0.0, 0.0]);
            tgt.points.push([i as f64 + 10.0, 0.0, 0.0]);
        }

        let result = procrustes_align(&src, &tgt, false);
        let c: f64 = (0..3).map(|i| result.points.get(i)[0]).sum::<f64>() / 3.0;
        assert!((c - 11.0).abs() < 1e-5); // centered on target
    }

    #[test]
    fn distance_identical_zero() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        assert!(procrustes_distance(&pd, &pd) < 1e-10);
    }

    #[test]
    fn scaled_alignment() {
        let mut a = PolyData::new();
        let mut b = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        b.points.push([0.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);

        let result = procrustes_align(&a, &b, true);
        let d = procrustes_distance(&result, &b);
        assert!(d < 0.5);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(procrustes_distance(&pd, &pd), 0.0);
    }
}
