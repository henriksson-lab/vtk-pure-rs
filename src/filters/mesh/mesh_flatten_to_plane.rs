//! Flatten mesh onto a best-fit plane.
use crate::data::PolyData;
pub fn flatten_to_best_fit_plane(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    // Covariance matrix
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        for a in 0..3 {
            for b in 0..3 {
                cov[a][b] += d[a] * d[b];
            }
        }
    }
    let (eigenvalues, eigenvectors) = jacobi_eigen_symmetric_3x3(cov);
    let mut axes = [0usize, 1, 2];
    axes.sort_by(|&a, &b| {
        eigenvalues[b]
            .partial_cmp(&eigenvalues[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let u_axis = eigenvectors[axes[0]];
    let v_axis = eigenvectors[axes[1]];

    let mut r = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        let u = d[0] * u_axis[0] + d[1] * u_axis[1] + d[2] * u_axis[2];
        let w = d[0] * v_axis[0] + d[1] * v_axis[1] + d[2] * v_axis[2];
        r.points.set(i, [u, w, 0.0]);
    }
    r
}

fn jacobi_eigen_symmetric_3x3(mut a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..24 {
        let mut p = 0;
        let mut q = 1;
        let mut max_offdiag = a[0][1].abs();
        for i in 0..3 {
            for j in i + 1..3 {
                let value = a[i][j].abs();
                if value > max_offdiag {
                    max_offdiag = value;
                    p = i;
                    q = j;
                }
            }
        }
        if max_offdiag <= 1e-15 {
            break;
        }

        let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = app - t * apq;
        a[q][q] = aqq + t * apq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                a[r][p] = c * arp - s * arq;
                a[p][r] = a[r][p];
                a[r][q] = s * arp + c * arq;
                a[q][r] = a[r][q];
            }
        }

        for row in &mut v {
            let vrp = row[p];
            let vrq = row[q];
            row[p] = c * vrp - s * vrq;
            row[q] = s * vrp + c * vrq;
        }
    }

    let eigenvalues = [a[0][0], a[1][1], a[2][2]];
    let eigenvectors = [
        [v[0][0], v[1][0], v[2][0]],
        [v[0][1], v[1][1], v[2][1]],
        [v[0][2], v[1][2], v[2][2]],
    ];
    (eigenvalues, eigenvectors)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.1], [1.0, 0.0, -0.1], [0.5, 1.0, 0.05]],
            vec![[0, 1, 2]],
        );
        let r = flatten_to_best_fit_plane(&m);
        for i in 0..3 {
            let p = r.points.get(i);
            assert!(p[2].abs() < 1e-5);
        }
    }
}
