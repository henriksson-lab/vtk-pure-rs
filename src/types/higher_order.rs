/// Lagrange basis function evaluation for higher-order cells.
///
/// These functions evaluate Lagrange interpolation basis functions
/// at parametric coordinates for various cell types.

/// Evaluate 1D Lagrange basis functions of given order at parameter t in [0,1].
///
/// Returns `order + 1` basis values that sum to 1.
pub fn lagrange_1d(order: usize, t: f64) -> Vec<f64> {
    if order == 0 {
        return vec![1.0];
    }
    let n = order + 1;
    let v = order as f64 * t;
    let mut basis = vec![1.0; n];
    for i in 0..n {
        for j in 0..n {
            if i != j {
                basis[i] *= (v - j as f64) / (i as f64 - j as f64);
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
    let basis = lagrange_curve_weights(order, t);
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
    lagrange_2d_quad_order([order, order], u, v)
}

/// Evaluate VTK-ordered 1D tensor Lagrange shape functions for a curve.
pub fn lagrange_curve_weights(order: usize, t: f64) -> Vec<f64> {
    let bu = lagrange_1d(order, t);
    if order <= 1 {
        return bu;
    }
    let mut weights = Vec::with_capacity(order + 1);
    weights.push(bu[0]);
    weights.push(bu[order]);
    weights.extend_from_slice(&bu[1..order]);
    weights
}

/// Evaluate VTK-ordered tensor-product Lagrange basis on a quad.
pub fn lagrange_2d_quad_order(order: [usize; 2], u: f64, v: f64) -> Vec<f64> {
    if order == [0, 0] {
        return vec![1.0];
    }
    let bu = lagrange_1d(order[0], u);
    let bv = lagrange_1d(order[1], v);
    let mut basis = Vec::with_capacity((order[0] + 1) * (order[1] + 1));

    basis.push(bu[0] * bv[0]);
    basis.push(bu[order[0]] * bv[0]);
    basis.push(bu[order[0]] * bv[order[1]]);
    basis.push(bu[0] * bv[order[1]]);

    let edge_count = 2 * (order[0].saturating_sub(1) + order[1].saturating_sub(1));
    basis.resize(4 + edge_count, 0.0);
    let mut sn = 4;
    let mut sn1 = sn + order[0].saturating_sub(1) + order[1].saturating_sub(1);

    for i in 1..order[0] {
        basis[sn] = bu[i] * bv[0];
        basis[sn1] = bu[i] * bv[order[1]];
        sn += 1;
        sn1 += 1;
    }
    for j in 1..order[1] {
        basis[sn] = bu[order[0]] * bv[j];
        basis[sn1] = bu[0] * bv[j];
        sn += 1;
        sn1 += 1;
    }

    for j in 1..order[1] {
        for i in 1..order[0] {
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
        let pts = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
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
    fn lagrange_curve_uses_vtk_point_order() {
        let weights = lagrange_curve_weights(2, 1.0);
        assert!(weights[0].abs() < 1e-12);
        assert!((weights[1] - 1.0).abs() < 1e-12);
        assert!(weights[2].abs() < 1e-12);
    }

    #[test]
    fn lagrange_quad_uses_vtk_point_order() {
        let weights = lagrange_2d_quad(2, 1.0, 1.0);
        assert!(weights[0].abs() < 1e-12);
        assert!(weights[1].abs() < 1e-12);
        assert!((weights[2] - 1.0).abs() < 1e-12);
        assert!(weights[3].abs() < 1e-12);
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
