//! Surface parameterization: conformal, harmonic, LSCM-like.

use crate::data::{AnyDataArray, DataArray, PolyData};

pub use crate::filters::mesh::parameterize_harmonic::harmonic_parameterize;

/// Projection-based parameterization: project onto best-fit plane.
pub fn planar_parameterize(mesh: &PolyData) -> PolyData {
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

    let mut covariance = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let q = [p[0] - cx, p[1] - cy, p[2] - cz];
        for r in 0..3 {
            for c in 0..3 {
                covariance[r][c] += q[r] * q[c];
            }
        }
    }
    for row in &mut covariance {
        for value in row {
            *value /= nf;
        }
    }

    let (eigenvalues, eigenvectors) = jacobi_eigen_symmetric_3x3(covariance);
    let mut axes = [0usize, 1, 2];
    axes.sort_by(|&a, &b| eigenvalues[b].total_cmp(&eigenvalues[a]));
    let axis_u = eigenvectors[axes[0]];
    let axis_v = eigenvectors[axes[1]];

    let mut u_vals = Vec::with_capacity(n);
    let mut v_vals = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        let q = [p[0] - cx, p[1] - cy, p[2] - cz];
        u_vals.push(dot(q, axis_u));
        v_vals.push(dot(q, axis_v));
    }

    let u_min = u_vals.iter().cloned().fold(f64::MAX, f64::min);
    let u_max = u_vals.iter().cloned().fold(f64::MIN, f64::max);
    let v_min = v_vals.iter().cloned().fold(f64::MAX, f64::min);
    let v_max = v_vals.iter().cloned().fold(f64::MIN, f64::max);
    let ur = (u_max - u_min).max(1e-15);
    let vr = (v_max - v_min).max(1e-15);

    let mut tc = Vec::with_capacity(n * 2);
    for i in 0..n {
        tc.push((u_vals[i] - u_min) / ur);
        tc.push((v_vals[i] - v_min) / vr);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", tc, 2)));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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
    fn planar() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = planar_parameterize(&mesh);
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn planar_uses_best_fit_plane_not_xy_only() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = planar_parameterize(&mesh);
        let arr = result.point_data().tcoords().unwrap();
        let mut uv0 = [0.0f64; 2];
        let mut uv2 = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut uv0);
        arr.tuple_as_f64(2, &mut uv2);
        assert!(
            (uv0[0] - uv2[0]).abs() > 1e-8 || (uv0[1] - uv2[1]).abs() > 1e-8,
            "points with identical XY collapse under XY projection but not under best-fit projection"
        );
    }

    #[test]
    fn harmonic_ignores_invalid_boundary_ids() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 99]);
        mesh.polys.push_cell(&[0, 1, 2]);

        let result = harmonic_parameterize(&mesh, 1);
        assert!(result.point_data().tcoords().is_some());
    }
}
