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

/// Compute principal axes of inertia via power iteration.
///
/// Returns (eigenvalues, eigenvectors) sorted by eigenvalue descending.
pub fn principal_axes(input: &PolyData) -> ([f64; 3], [[f64; 3]; 3]) {
    let t = inertia_tensor(input);

    // Power iteration for largest eigenvalue
    let v1 = power_iter(&t);
    let e1 = rayleigh(&t, &v1);

    // Deflate
    let mut t2 = t;
    for r in 0..3 {
        for c in 0..3 {
            t2[r][c] -= e1 * v1[r] * v1[c];
        }
    }
    let v2 = power_iter(&t2);
    let e2 = rayleigh(&t2, &v2);

    // Third = cross product
    let v3 = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    let mut t3 = t2;
    for r in 0..3 {
        for c in 0..3 {
            t3[r][c] -= e2 * v2[r] * v2[c];
        }
    }
    let e3 = rayleigh(&t, &v3);

    ([e1, e2, e3], [v1, v2, v3])
}

fn power_iter(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let s = 1.0 / 3.0f64.sqrt();
    let mut v = [s, s, s];
    for _ in 0..50 {
        let mut nv = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                nv[r] += m[r][c] * v[c];
            }
        }
        let l = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if l > 1e-15 {
            v = [nv[0] / l, nv[1] / l, nv[2] / l];
        }
    }
    v
}

fn rayleigh(m: &[[f64; 3]; 3], v: &[f64; 3]) -> f64 {
    let mut mv = [0.0; 3];
    for r in 0..3 {
        for c in 0..3 {
            mv[r] += m[r][c] * v[c];
        }
    }
    v[0] * mv[0] + v[1] * mv[1] + v[2] * mv[2]
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
        assert!(dot.abs() < 0.3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let t = inertia_tensor(&pd);
        assert_eq!(t, [[0.0; 3]; 3]);
    }
}
