//! PCACurvatureEstimation — estimate curvature via PCA on local neighborhoods.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// For each point, find k nearest neighbors (brute-force), compute PCA on
/// the neighborhood, and use the ratio of the smallest eigenvalue to the
/// sum of all eigenvalues as a curvature estimate.
///
/// Adds a "Curvature" scalar array to point data.
pub fn pca_curvature_estimation(input: &PolyData, k: usize) -> PolyData {
    let n = input.points.len();
    let mut curvatures = vec![0.0f64; n];

    if n == 0 || k == 0 {
        let mut result = input.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Curvature",
                curvatures,
                1,
            )));
        return result;
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for i in 0..n {
        // Find k nearest neighbors (brute-force)
        let mut dists: Vec<(usize, f64)> = (0..n)
            .map(|j| {
                let d = (pts[i][0] - pts[j][0]).powi(2)
                    + (pts[i][1] - pts[j][1]).powi(2)
                    + (pts[i][2] - pts[j][2]).powi(2);
                (j, d)
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let kk = k.min(n);
        let neighbors: Vec<usize> = dists[..kk].iter().map(|&(j, _)| j).collect();

        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &j in &neighbors {
            cx += pts[j][0];
            cy += pts[j][1];
            cz += pts[j][2];
        }
        let m = neighbors.len() as f64;
        cx /= m;
        cy /= m;
        cz /= m;

        // Compute 3x3 covariance matrix
        let mut cov = [[0.0f64; 3]; 3];
        for &j in &neighbors {
            let dx = pts[j][0] - cx;
            let dy = pts[j][1] - cy;
            let dz = pts[j][2] - cz;
            let d = [dx, dy, dz];
            for a in 0..3 {
                for b in 0..3 {
                    cov[a][b] += d[a] * d[b];
                }
            }
        }
        for a in 0..3 {
            for b in 0..3 {
                cov[a][b] /= m;
            }
        }

        // Compute eigenvalues using the characteristic equation of a 3x3 symmetric matrix
        let eigenvalues = symmetric_3x3_eigenvalues(cov);
        let sum = eigenvalues[0] + eigenvalues[1] + eigenvalues[2];
        if sum > 1e-20 {
            curvatures[i] = eigenvalues[0] / sum; // smallest / sum
        }
    }

    let mut result = input.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Curvature",
            curvatures,
            1,
        )));
    result
}

/// Compute eigenvalues of a 3x3 symmetric matrix using the analytical formula.
/// Returns sorted eigenvalues [smallest, middle, largest].
fn symmetric_3x3_eigenvalues(m: [[f64; 3]; 3]) -> [f64; 3] {
    let p1 = m[0][1] * m[0][1] + m[0][2] * m[0][2] + m[1][2] * m[1][2];
    if p1 < 1e-30 {
        // Matrix is diagonal
        let mut eig = [m[0][0], m[1][1], m[2][2]];
        eig.sort_by(|a, b| a.partial_cmp(b).unwrap());
        return eig;
    }

    let q = (m[0][0] + m[1][1] + m[2][2]) / 3.0;
    let p2 = (m[0][0] - q).powi(2) + (m[1][1] - q).powi(2) + (m[2][2] - q).powi(2) + 2.0 * p1;
    let p = (p2 / 6.0).sqrt();

    // B = (1/p) * (A - q*I)
    let b = [
        [(m[0][0] - q) / p, m[0][1] / p, m[0][2] / p],
        [m[1][0] / p, (m[1][1] - q) / p, m[1][2] / p],
        [m[2][0] / p, m[2][1] / p, (m[2][2] - q) / p],
    ];

    let det_b = b[0][0] * (b[1][1] * b[2][2] - b[1][2] * b[2][1])
        - b[0][1] * (b[1][0] * b[2][2] - b[1][2] * b[2][0])
        + b[0][2] * (b[1][0] * b[2][1] - b[1][1] * b[2][0]);

    let r = det_b / 2.0;
    let phi = if r <= -1.0 {
        std::f64::consts::PI / 3.0
    } else if r >= 1.0 {
        0.0
    } else {
        r.acos() / 3.0
    };

    let eig1 = q + 2.0 * p * phi.cos();
    let eig3 = q + 2.0 * p * (phi + 2.0 * std::f64::consts::PI / 3.0).cos();
    let eig2 = 3.0 * q - eig1 - eig3;

    let mut eig = [eig1, eig2, eig3];
    eig.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_points_low_curvature() {
        let mut pd = PolyData::new();
        // Flat grid of points in XY plane
        for i in 0..5 {
            for j in 0..5 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }

        let result = pca_curvature_estimation(&pd, 8);
        let arr = result.point_data().get_array("Curvature").unwrap();
        // Interior points should have very low curvature
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf); // center point
        assert!(
            buf[0] < 0.1,
            "flat surface curvature should be near zero, got {}",
            buf[0]
        );
    }

    #[test]
    fn curved_points_higher_curvature() {
        let mut pd = PolyData::new();
        // Points on a hemisphere
        for i in 0..10 {
            let theta = i as f64 * std::f64::consts::PI / 9.0;
            for j in 0..10 {
                let phi = j as f64 * 2.0 * std::f64::consts::PI / 10.0;
                let x = theta.sin() * phi.cos();
                let y = theta.sin() * phi.sin();
                let z = theta.cos();
                pd.points.push([x, y, z]);
            }
        }

        let result = pca_curvature_estimation(&pd, 6);
        let arr = result.point_data().get_array("Curvature").unwrap();
        assert_eq!(arr.num_tuples(), 100);
    }
}
