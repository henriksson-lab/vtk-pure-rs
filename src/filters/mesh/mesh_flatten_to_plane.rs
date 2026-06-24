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
    // Power iteration for normal (smallest eigenvector)
    let mut v = [0.0, 0.0, 1.0];
    for _ in 0..50 {
        let mv = [
            cov[0][0] * v[0] + cov[0][1] * v[1] + cov[0][2] * v[2],
            cov[1][0] * v[0] + cov[1][1] * v[1] + cov[1][2] * v[2],
            cov[2][0] * v[0] + cov[2][1] * v[1] + cov[2][2] * v[2],
        ];
        let l = (mv[0] * mv[0] + mv[1] * mv[1] + mv[2] * mv[2])
            .sqrt()
            .max(1e-15);
        v = [mv[0] / l, mv[1] / l, mv[2] / l];
    }
    // v is largest eigenvector; normal is perpendicular
    let mut v2 = if v[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let d = v2[0] * v[0] + v2[1] * v[1] + v2[2] * v[2];
    v2 = [v2[0] - d * v[0], v2[1] - d * v[1], v2[2] - d * v[2]];
    let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2])
        .sqrt()
        .max(1e-15);
    v2 = [v2[0] / l2, v2[1] / l2, v2[2] / l2];
    // Use v and v2 as 2D axes, project
    let mut r = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - cx, p[1] - cy, p[2] - cz];
        let u = d[0] * v[0] + d[1] * v[1] + d[2] * v[2];
        let w = d[0] * v2[0] + d[1] * v2[1] + d[2] * v2[2];
        r.points.set(i, [u, w, 0.0]);
    }
    r
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
