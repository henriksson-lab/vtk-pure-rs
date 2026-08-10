use crate::data::PolyData;

/// Moment of inertia tensor for a point set.
///
/// Returns the 3x3 symmetric inertia tensor assuming unit mass per point.
pub fn inertia_tensor(input: &PolyData) -> [[f64; 3]; 3] {
    let n = input.points.len();
    if n == 0 {
        return [[0.0; 3]; 3];
    }

    // Centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    let mut tensor = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = input.points.get(i);
        let x = p[0] - cx;
        let y = p[1] - cy;
        let z = p[2] - cz;
        tensor[0][0] += y * y + z * z;
        tensor[0][1] -= x * y;
        tensor[0][2] -= x * z;
        tensor[1][0] -= x * y;
        tensor[1][1] += x * x + z * z;
        tensor[1][2] -= y * z;
        tensor[2][0] -= x * z;
        tensor[2][1] -= y * z;
        tensor[2][2] += x * x + y * y;
    }
    tensor
}

/// Compute principal axes of inertia.
///
/// Returns (eigenvalues, eigenvectors) sorted by inertia eigenvalue descending.
///
/// The eigen-decomposition itself is shared with
/// [`crate::filters::mesh::principal_axes::principal_axes`], which diagonalises the
/// *covariance* matrix the way `vtkOBBTree::ComputeOBB` does. The inertia tensor of a
/// unit-mass point set is `I = n * (trace(C) * Id - C)`, so it has the same
/// eigenvectors as the covariance matrix `C` with the eigenvalue ordering reversed:
/// the widest spread direction carries the *smallest* moment of inertia.
pub fn principal_axes(input: &PolyData) -> ([f64; 3], [[f64; 3]; 3]) {
    let n = input.points.len() as f64;
    let (_, axes, spread) = crate::filters::mesh::principal_axes::principal_axes(input);
    let trace = spread[0] + spread[1] + spread[2];
    (
        [
            n * (trace - spread[2]),
            n * (trace - spread[1]),
            n * (trace - spread[0]),
        ],
        [axes[2], axes[1], axes[0]],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric_tensor() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);

        let t = inertia_tensor(&pd);
        assert!((t[0][1] - t[1][0]).abs() < 1e-10); // symmetric
        assert!((t[0][2] - t[2][0]).abs() < 1e-10);
    }

    #[test]
    fn principal_axes_orthogonal() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        for j in 0..5 {
            pd.points.push([0.0, j as f64, 0.0]);
        }

        let (evals, evecs) = principal_axes(&pd);
        assert!(evals[0] >= evals[1]); // sorted descending
                                       // Eigenvectors should be roughly orthogonal
        let dot = evecs[0][0] * evecs[1][0] + evecs[0][1] * evecs[1][1] + evecs[0][2] * evecs[1][2];
        assert!(dot.abs() < 1e-10);
    }

    #[test]
    fn principal_axes_degenerate_are_orthonormal() {
        let pd = PolyData::new();
        let (evals, evecs) = principal_axes(&pd);
        assert_eq!(evals, [0.0; 3]);
        for axis in &evecs {
            let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let t = inertia_tensor(&pd);
        assert_eq!(t, [[0.0; 3]; 3]);
    }
}
