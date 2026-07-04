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
    jacobi_eigen_symmetric(inertia_tensor(input))
}

fn jacobi_eigen_symmetric(mut a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max = a[0][1].abs();
        for &(r, c) in &[(0usize, 2usize), (1usize, 2usize)] {
            if a[r][c].abs() > max {
                max = a[r][c].abs();
                p = r;
                q = c;
            }
        }
        if max < 1e-12 {
            break;
        }

        let theta = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let c = theta.cos();
        let s = theta.sin();
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r == p || r == q {
                continue;
            }
            let arp = a[r][p];
            let arq = a[r][q];
            a[r][p] = c * arp - s * arq;
            a[p][r] = a[r][p];
            a[r][q] = s * arp + c * arq;
            a[q][r] = a[r][q];
        }

        for row in &mut v {
            let vrp = row[p];
            let vrq = row[q];
            row[p] = c * vrp - s * vrq;
            row[q] = s * vrp + c * vrq;
        }
    }

    let mut pairs = [
        (a[0][0], normalize([v[0][0], v[1][0], v[2][0]])),
        (a[1][1], normalize([v[0][1], v[1][1], v[2][1]])),
        (a[2][2], normalize([v[0][2], v[1][2], v[2][2]])),
    ];
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    (
        [pairs[0].0, pairs[1].0, pairs[2].0],
        [pairs[0].1, pairs[1].1, pairs[2].1],
    )
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
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
