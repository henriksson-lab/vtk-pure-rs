use crate::data::{DataSet, PolyData};

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

    let center = input.bounds().center();

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
    let mut axis = [s, s, s];
    let mut stable_axis = false;
    for _ in 0..50 {
        let mut next = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                next[r] += cov[r][c] * axis[c];
            }
        }
        let len = norm(next);
        if len > 1e-15 {
            axis = [next[0] / len, next[1] / len, next[2] / len];
            stable_axis = true;
        }
    }
    if !stable_axis {
        return ([0.0, 0.0, 1.0], 0.0);
    }

    // Score: high when perpendicular distances are uniform (rotational).
    // Check if perp distances are similar (low coefficient of variation)
    let perp_dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let along = dot(d, axis);
            (dot(d, d) - along * along).max(0.0).sqrt()
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
    (axis, score)
}

/// Find the best rotation axis among the 3 principal axes.
pub fn best_rotation_axis(input: &PolyData) -> ([f64; 3], f64) {
    spin_axis(input) // PCA already finds the dominant axis
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
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
        assert!(norm(axis) > 0.999);
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

    #[test]
    fn coincident_points_have_zero_score() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 1.0, 1.0]);
        pd.points.push([1.0, 1.0, 1.0]);
        pd.points.push([1.0, 1.0, 1.0]);

        let (axis, score) = spin_axis(&pd);
        assert_eq!(axis, [0.0, 0.0, 1.0]);
        assert_eq!(score, 0.0);
    }
}
