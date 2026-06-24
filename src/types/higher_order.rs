/// Lagrange basis function evaluation for higher-order cells.
///
/// These functions evaluate Lagrange interpolation basis functions
/// at parametric coordinates for various cell types.

/// Evaluate 1D Lagrange basis functions of given order at parameter t in [0,1].
///
/// Returns `order + 1` basis values that sum to 1.
pub fn lagrange_1d(order: usize, t: f64) -> Vec<f64> {
    let n = order + 1;
    let nodes: Vec<f64> = (0..n).map(|i| i as f64 / order as f64).collect();
    let mut basis = vec![1.0; n];
    for i in 0..n {
        for j in 0..n {
            if i != j {
                basis[i] *= (t - nodes[j]) / (nodes[i] - nodes[j]);
            }
        }
    }
    basis
}

/// Evaluate a point on a Lagrange curve at parameter t in [0,1].
///
/// `control_points` has `order + 1` points.
pub fn eval_lagrange_curve(control_points: &[[f64; 3]], t: f64) -> [f64; 3] {
    let order = control_points.len() - 1;
    let basis = lagrange_1d(order, t);
    let mut result = [0.0; 3];
    for (i, b) in basis.iter().enumerate() {
        result[0] += b * control_points[i][0];
        result[1] += b * control_points[i][1];
        result[2] += b * control_points[i][2];
    }
    result
}

/// Evaluate a point on a Bernstein-Bezier curve at parameter t in [0,1].
///
/// `control_points` has `order + 1` points.
pub fn eval_bezier_curve(control_points: &[[f64; 3]], t: f64) -> [f64; 3] {
    let n = control_points.len() - 1;
    let basis = bernstein_basis(n, t);
    let mut result = [0.0; 3];
    for (i, b) in basis.iter().enumerate() {
        result[0] += b * control_points[i][0];
        result[1] += b * control_points[i][1];
        result[2] += b * control_points[i][2];
    }
    result
}

/// Evaluate Bernstein basis polynomials of degree n at parameter t.
pub fn bernstein_basis(n: usize, t: f64) -> Vec<f64> {
    let mut basis = vec![0.0; n + 1];
    basis[0] = 1.0;
    let s = 1.0 - t;
    // de Casteljau-style evaluation for numerical stability
    for j in 1..=n {
        let mut saved = 0.0;
        for k in 0..j {
            let temp = basis[k];
            basis[k] = saved + s * temp;
            saved = t * temp;
        }
        basis[j] = saved;
    }
    basis
}

/// Evaluate 2D tensor-product Lagrange basis on a quad at (u, v) in [0,1]^2.
///
/// Returns basis values for an `(order+1) x (order+1)` grid of nodes.
pub fn lagrange_2d_quad(order: usize, u: f64, v: f64) -> Vec<f64> {
    let bu = lagrange_1d(order, u);
    let bv = lagrange_1d(order, v);
    let n = order + 1;
    let mut basis = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            basis.push(bu[i] * bv[j]);
        }
    }
    basis
}

/// Evaluate a point on a Lagrange quadrilateral at (u, v) in [0,1]^2.
pub fn eval_lagrange_quad(control_points: &[[f64; 3]], order: usize, u: f64, v: f64) -> [f64; 3] {
    let basis = lagrange_2d_quad(order, u, v);
    let mut result = [0.0; 3];
    for (i, b) in basis.iter().enumerate() {
        if i < control_points.len() {
            result[0] += b * control_points[i][0];
            result[1] += b * control_points[i][1];
            result[2] += b * control_points[i][2];
        }
    }
    result
}

/// Tessellate a higher-order curve into line segments.
///
/// Returns a list of points sampled uniformly along the curve.
pub fn tessellate_lagrange_curve(
    control_points: &[[f64; 3]],
    num_segments: usize,
) -> Vec<[f64; 3]> {
    (0..=num_segments)
        .map(|i| {
            let t = i as f64 / num_segments as f64;
            eval_lagrange_curve(control_points, t)
        })
        .collect()
}

/// Tessellate a Bezier curve into line segments.
pub fn tessellate_bezier_curve(control_points: &[[f64; 3]], num_segments: usize) -> Vec<[f64; 3]> {
    (0..=num_segments)
        .map(|i| {
            let t = i as f64 / num_segments as f64;
            eval_bezier_curve(control_points, t)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lagrange_linear_partition_of_unity() {
        let basis = lagrange_1d(1, 0.5);
        let sum: f64 = basis.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn lagrange_quadratic_endpoints() {
        let basis = lagrange_1d(2, 0.0);
        assert!((basis[0] - 1.0).abs() < 1e-12);
        assert!(basis[1].abs() < 1e-12);
        assert!(basis[2].abs() < 1e-12);
    }

    #[test]
    fn bezier_linear_is_lerp() {
        let pts = [[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]];
        let mid = eval_bezier_curve(&pts, 0.5);
        assert!((mid[0] - 1.0).abs() < 1e-12);
        assert!((mid[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn bezier_quadratic_midpoint() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [2.0, 0.0, 0.0]];
        let mid = eval_bezier_curve(&pts, 0.5);
        // Quadratic Bezier at t=0.5: (1/4)*P0 + (1/2)*P1 + (1/4)*P2
        assert!((mid[0] - 1.0).abs() < 1e-12);
        assert!((mid[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn bernstein_partition_of_unity() {
        for n in 1..=5 {
            let basis = bernstein_basis(n, 0.37);
            let sum: f64 = basis.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "n={n}, sum={sum}");
        }
    }

    #[test]
    fn lagrange_curve_endpoints() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]];
        let start = eval_lagrange_curve(&pts, 0.0);
        let end = eval_lagrange_curve(&pts, 1.0);
        assert!((start[0]).abs() < 1e-12);
        assert!((end[0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn lagrange_2d_quad_corners() {
        let basis = lagrange_2d_quad(1, 0.0, 0.0);
        assert!((basis[0] - 1.0).abs() < 1e-12); // bottom-left
        assert!(basis[1].abs() < 1e-12);
    }

    #[test]
    fn tessellate_curve() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let tessellated = tessellate_lagrange_curve(&pts, 4);
        assert_eq!(tessellated.len(), 5);
        assert!((tessellated[0][0]).abs() < 1e-12);
        assert!((tessellated[4][0] - 1.0).abs() < 1e-12);
    }
}
