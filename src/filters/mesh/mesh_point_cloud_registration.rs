//! Simple point cloud registration (rigid alignment via centroid + PCA).
use crate::data::PolyData;
pub fn align_centroids(source: &PolyData, target: &PolyData) -> PolyData {
    let sc = centroid(source);
    let tc = centroid(target);
    let offset = [tc[0] - sc[0], tc[1] - sc[1], tc[2] - sc[2]];
    let mut r = source.clone();
    for i in 0..r.points.len() {
        let p = r.points.get(i);
        r.points
            .set(i, [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]]);
    }
    r
}
pub fn registration_error(a: &PolyData, b: &PolyData) -> f64 {
    let na = a.points.len();
    if na == 0 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..na {
        let pa = a.points.get(i);
        let mut best = f64::INFINITY;
        for j in 0..b.points.len() {
            let pb = b.points.get(j);
            let d = (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
            best = best.min(d);
        }
        total += best.sqrt();
    }
    total / na as f64
}
fn centroid(mesh: &PolyData) -> [f64; 3] {
    let n = mesh.points.len();
    if n == 0 {
        return [0.0; 3];
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let nf = n as f64;
    [c[0] / nf, c[1] / nf, c[2] / nf]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_align() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[10.0, 10.0, 0.0], [11.0, 10.0, 0.0], [10.5, 11.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = align_centroids(&a, &b);
        let c = centroid(&r);
        let cb = centroid(&b);
        assert!((c[0] - cb[0]).abs() < 1e-10);
    }
    #[test]
    fn test_error() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let e = registration_error(&a, &a);
        assert!(e < 1e-10);
    }
}
