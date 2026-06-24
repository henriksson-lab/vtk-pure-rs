use crate::data::{KdTree, Points, PolyData};

/// Rigid registration of two point sets using iterative closest point (simple version).
///
/// Aligns `source` to `target` by iteratively finding correspondences
/// and computing best-fit translation. Returns the aligned source.
/// Simpler than the full `icp` filter — translation only, no rotation.
pub fn translate_to_match(source: &PolyData, target: &PolyData, iterations: usize) -> PolyData {
    let n_src = source.points.len();
    let n_tgt = target.points.len();
    if n_src == 0 || n_tgt == 0 {
        return source.clone();
    }

    let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let mut pts: Vec<[f64; 3]> = (0..n_src).map(|i| source.points.get(i)).collect();

    for _ in 0..iterations {
        // Find average displacement to closest target points
        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;
        let mut count = 0;

        for p in &pts {
            if let Some((idx, _)) = tree.nearest(*p) {
                let t = tgt_pts[idx];
                dx += t[0] - p[0];
                dy += t[1] - p[1];
                dz += t[2] - p[2];
                count += 1;
            }
        }

        if count == 0 {
            break;
        }
        let nf = count as f64;
        dx /= nf;
        dy /= nf;
        dz /= nf;

        // Apply translation
        for p in &mut pts {
            p[0] += dx;
            p[1] += dy;
            p[2] += dz;
        }

        // Stop if displacement is tiny
        if dx * dx + dy * dy + dz * dz < 1e-20 {
            break;
        }
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = source.clone();
    pd.points = points;
    pd
}

/// Compute the Hausdorff distance after alignment.
pub fn registration_error(source: &PolyData, target: &PolyData) -> f64 {
    let n_src = source.points.len();
    let n_tgt = target.points.len();
    if n_src == 0 || n_tgt == 0 {
        return 0.0;
    }

    let tgt_pts: Vec<[f64; 3]> = (0..n_tgt).map(|i| target.points.get(i)).collect();
    let tree = KdTree::build(&tgt_pts);

    let mut max_d = 0.0f64;
    for i in 0..n_src {
        if let Some((_, d2)) = tree.nearest(source.points.get(i)) {
            max_d = max_d.max(d2.sqrt());
        }
    }
    max_d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_to_target() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.points.push([1.0, 0.0, 0.0]);

        let mut tgt = PolyData::new();
        tgt.points.push([10.0, 0.0, 0.0]);
        tgt.points.push([11.0, 0.0, 0.0]);

        let result = translate_to_match(&src, &tgt, 10);
        let p = result.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn already_aligned() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        let mut tgt = PolyData::new();
        tgt.points.push([0.0, 0.0, 0.0]);

        let result = translate_to_match(&src, &tgt, 5);
        let p = result.points.get(0);
        assert!(p[0].abs() < 1e-10);
    }

    #[test]
    fn error_metric() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([3.0, 4.0, 0.0]);

        let err = registration_error(&a, &b);
        assert!((err - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let src = PolyData::new();
        let tgt = PolyData::new();
        let result = translate_to_match(&src, &tgt, 5);
        assert_eq!(result.points.len(), 0);
    }
}
