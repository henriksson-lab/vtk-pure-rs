//! Simple ICP (Iterative Closest Point) alignment.
use crate::data::PolyData;
pub fn icp_align(source: &PolyData, target: &PolyData, max_iterations: usize) -> PolyData {
    let sn = source.points.len();
    let tn = target.points.len();
    if sn == 0 || tn == 0 {
        return source.clone();
    }
    let tpts: Vec<[f64; 3]> = (0..tn).map(|i| target.points.get(i)).collect();
    let mut r = source.clone();
    for _ in 0..max_iterations {
        // Find correspondences (nearest points)
        let spts: Vec<[f64; 3]> = (0..sn).map(|i| r.points.get(i)).collect();
        let corr: Vec<usize> = (0..sn)
            .map(|i| {
                let p = spts[i];
                (0..tn)
                    .min_by(|&a, &b| {
                        let da = (p[0] - tpts[a][0]).powi(2)
                            + (p[1] - tpts[a][1]).powi(2)
                            + (p[2] - tpts[a][2]).powi(2);
                        let db = (p[0] - tpts[b][0]).powi(2)
                            + (p[1] - tpts[b][1]).powi(2)
                            + (p[2] - tpts[b][2]).powi(2);
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap()
            })
            .collect();
        // Compute centroids
        let mut sc = [0.0, 0.0, 0.0];
        let mut tc = [0.0, 0.0, 0.0];
        for i in 0..sn {
            sc[0] += spts[i][0];
            sc[1] += spts[i][1];
            sc[2] += spts[i][2];
            tc[0] += tpts[corr[i]][0];
            tc[1] += tpts[corr[i]][1];
            tc[2] += tpts[corr[i]][2];
        }
        let nf = sn as f64;
        for j in 0..3 {
            sc[j] /= nf;
            tc[j] /= nf;
        }
        // Apply translation (simplified: just centroid alignment per iteration)
        let dx = tc[0] - sc[0];
        let dy = tc[1] - sc[1];
        let dz = tc[2] - sc[2];
        for i in 0..sn {
            let p = r.points.get(i);
            r.points
                .set(i, [p[0] + dx * 0.5, p[1] + dy * 0.5, p[2] + dz * 0.5]);
        }
    }
    r
}
pub fn icp_error(source: &PolyData, target: &PolyData) -> f64 {
    let sn = source.points.len();
    if sn == 0 {
        return 0.0;
    }
    let tn = target.points.len();
    if tn == 0 {
        return f64::INFINITY;
    }
    let mut total = 0.0;
    for i in 0..sn {
        let p = source.points.get(i);
        let mut best = f64::INFINITY;
        for j in 0..tn {
            let q = target.points.get(j);
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            best = best.min(d);
        }
        total += best.sqrt();
    }
    total / sn as f64
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_align() {
        let src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let tgt = PolyData::from_triangles(
            vec![[5.0, 5.0, 0.0], [6.0, 5.0, 0.0], [5.5, 6.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let aligned = icp_align(&src, &tgt, 20);
        let err_before = icp_error(&src, &tgt);
        let err_after = icp_error(&aligned, &tgt);
        assert!(err_after < err_before);
    }
    #[test]
    fn test_error() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(icp_error(&a, &a) < 1e-10);
    }
}
