use crate::data::{Points, PolyData};

/// Align two point clouds by matching their principal axes.
///
/// Computes PCA of both sets, aligns their centroids, then rotates
/// to align dominant axes. Rough alignment without ICP.
pub fn pca_align(source: &PolyData, target: &PolyData) -> PolyData {
    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return source.clone();
    }

    // Compute centroids
    let sc = centroid(source);
    let tc = centroid(target);

    // Compute dominant axis for each
    let sa = dominant_axis(source, sc);
    let ta = dominant_axis(target, tc);

    // Compute rotation from sa to ta using Rodrigues
    let dot = (sa[0] * ta[0] + sa[1] * ta[1] + sa[2] * ta[2]).clamp(-1.0, 1.0);
    let cross = [
        sa[1] * ta[2] - sa[2] * ta[1],
        sa[2] * ta[0] - sa[0] * ta[2],
        sa[0] * ta[1] - sa[1] * ta[0],
    ];
    let sin_a = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let half_turn_axis = if sin_a <= 1e-10 && dot < 0.0 {
        Some(perpendicular_axis(sa))
    } else {
        None
    };

    let mut points = Points::<f64>::new();
    for i in 0..ns {
        let p = source.points.get(i);
        let d = [p[0] - sc[0], p[1] - sc[1], p[2] - sc[2]];

        let rotated = if sin_a > 1e-10 {
            let k = [cross[0] / sin_a, cross[1] / sin_a, cross[2] / sin_a];
            rotate_rodrigues(d, k, dot, sin_a)
        } else if let Some(k) = half_turn_axis {
            rotate_rodrigues(d, k, -1.0, 0.0)
        } else {
            d
        };

        points.push([rotated[0] + tc[0], rotated[1] + tc[1], rotated[2] + tc[2]]);
    }

    let mut pd = source.clone();
    pd.points = points;
    pd
}

fn centroid(pd: &PolyData) -> [f64; 3] {
    let n = pd.points.len();
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = pd.points.get(i);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let nf = n as f64;
    [c[0] / nf, c[1] / nf, c[2] / nf]
}

fn dominant_axis(pd: &PolyData, center: [f64; 3]) -> [f64; 3] {
    let n = pd.points.len();
    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..n {
        let p = pd.points.get(i);
        let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }
    let s = 1.0 / 3.0f64.sqrt();
    let mut v = [s, s, s];
    for _ in 0..50 {
        let mut nv = [0.0; 3];
        for r in 0..3 {
            for c in 0..3 {
                nv[r] += cov[r][c] * v[c];
            }
        }
        let l = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if l > 1e-15 {
            v = [nv[0] / l, nv[1] / l, nv[2] / l];
        }
    }
    v
}

fn rotate_rodrigues(d: [f64; 3], k: [f64; 3], cos_a: f64, sin_a: f64) -> [f64; 3] {
    let kdot = k[0] * d[0] + k[1] * d[1] + k[2] * d[2];
    let kcross = [
        k[1] * d[2] - k[2] * d[1],
        k[2] * d[0] - k[0] * d[2],
        k[0] * d[1] - k[1] * d[0],
    ];
    [
        d[0] * cos_a + kcross[0] * sin_a + k[0] * kdot * (1.0 - cos_a),
        d[1] * cos_a + kcross[1] * sin_a + k[1] * kdot * (1.0 - cos_a),
        d[2] * cos_a + kcross[2] * sin_a + k[2] * kdot * (1.0 - cos_a),
    ]
}

fn perpendicular_axis(v: [f64; 3]) -> [f64; 3] {
    let basis = if v[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let axis = [
        v[1] * basis[2] - v[2] * basis[1],
        v[2] * basis[0] - v[0] * basis[2],
        v[0] * basis[1] - v[1] * basis[0],
    ];
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len > 1e-15 {
        [axis[0] / len, axis[1] / len, axis[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_translated() {
        let mut src = PolyData::new();
        for i in 0..5 {
            src.points.push([i as f64, 0.0, 0.0]);
        }
        let mut tgt = PolyData::new();
        for i in 0..5 {
            tgt.points.push([i as f64 + 10.0, 0.0, 0.0]);
        }

        let result = pca_align(&src, &tgt);
        // Centroid should be near target centroid
        let rc = centroid(&result);
        let tc = centroid(&tgt);
        assert!((rc[0] - tc[0]).abs() < 1.0);
    }

    #[test]
    fn same_mesh_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = pca_align(&pd, &pd);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(pca_align(&pd, &pd).points.len(), 0);
    }

    #[test]
    fn aligns_opposite_principal_axis() {
        let mut src = PolyData::new();
        src.points.push([-2.0, 0.0, 0.0]);
        src.points.push([-1.0, 0.0, 0.0]);
        src.points.push([0.0, 0.0, 0.0]);
        let mut tgt = PolyData::new();
        tgt.points.push([2.0, 0.0, 0.0]);
        tgt.points.push([1.0, 0.0, 0.0]);
        tgt.points.push([0.0, 0.0, 0.0]);

        let result = pca_align(&src, &tgt);
        for i in 0..src.points.len() {
            let p = result.points.get(i);
            let q = tgt.points.get(i);
            assert!((p[0] - q[0]).abs() < 1e-10, "point {i}: {p:?} != {q:?}");
        }
    }
}
