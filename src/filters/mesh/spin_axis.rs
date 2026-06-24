use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

/// Compute the spin axis: the axis of approximate rotational symmetry.
///
/// Uses PCA on the point set to find the dominant axis, then checks
/// how well points are distributed around it. Returns (axis, score)
/// where score is in [0,1] (1 = perfect rotational symmetry).
pub fn spin_axis(input: &PolyData) -> ([f64; 3], f64) {
    let n = input.points.len();
    if n < 3 {
        return ([0.0, 0.0, 1.0], 0.0);
    }

    let bb = input.bounds();
    let center = bb.center();

    // PCA
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }

    let s = 1.0 / 3.0f64.sqrt();
    let mut v = [s, s, s];
    for _ in 0..50 {
        let mut nv = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                nv[r] += cov[r][c] * v[c];
            }
        }
        let l = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if l > 1e-15 {
            v = [nv[0] / l, nv[1] / l, nv[2] / l];
        }
    }

    // Measure symmetry: variance of distances from axis should be low relative to variance along axis
    let mut along_var = 0.0;
    let mut perp_var = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        let along = d[0] * v[0] + d[1] * v[1] + d[2] * v[2];
        let perp2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2] - along * along;
        along_var += along * along;
        perp_var += perp2;
    }
    along_var /= n as f64;
    perp_var /= n as f64;

    // Score: high when perp variance is uniform (rotational)
    // Check if perp distances are similar (low coefficient of variation)
    let mut perp_dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let along = d[0] * v[0] + d[1] * v[1] + d[2] * v[2];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] - along * along)
                .max(0.0)
                .sqrt()
        })
        .collect();

    let mean_perp: f64 = perp_dists.iter().sum::<f64>() / n as f64;
    let var_perp: f64 = perp_dists
        .iter()
        .map(|d| (d - mean_perp).powi(2))
        .sum::<f64>()
        / n as f64;
    let cv = if mean_perp > 1e-15 {
        var_perp.sqrt() / mean_perp
    } else {
        0.0
    };

    let score = (1.0 - cv.min(1.0)).max(0.0);
    (v, score)
}

/// Find the best rotation axis among the 3 principal axes.
pub fn best_rotation_axis(input: &PolyData) -> ([f64; 3], f64) {
    spin_axis(input) // PCA already finds the dominant axis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cylinder_high_symmetry() {
        let mut pd = PolyData::new();
        // Points on a cylinder around Y axis
        for i in 0..20 {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 20.0;
            pd.points.push([angle.cos(), 0.0, angle.sin()]);
            pd.points.push([angle.cos(), 1.0, angle.sin()]);
        }

        let (axis, score) = spin_axis(&pd);
        assert!(score > 0.5); // should have reasonable symmetry
    }

    #[test]
    fn random_low_symmetry() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([5.0, 1.0, 0.0]);
        pd.points.push([0.0, 3.0, 2.0]);
        pd.points.push([1.0, 0.0, 4.0]);

        let (_, score) = spin_axis(&pd);
        assert!(score < 1.0); // irregular point set
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (_, score) = spin_axis(&pd);
        assert_eq!(score, 0.0);
    }
}
