//! Interpolate between two meshes (morph).
use crate::data::PolyData;
pub fn interpolate_meshes(a: &PolyData, b: &PolyData, t: f64) -> PolyData {
    let n = a.points.len().min(b.points.len());
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    let mut r = a.clone();
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        r.points.set(
            i,
            [
                pa[0] * s + pb[0] * t,
                pa[1] * s + pb[1] * t,
                pa[2] * s + pb[2] * t,
            ],
        );
    }
    r
}
pub fn interpolate_sequence(a: &PolyData, b: &PolyData, steps: usize) -> Vec<PolyData> {
    (0..=steps)
        .map(|i| interpolate_meshes(a, b, i as f64 / steps.max(1) as f64))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lerp() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = interpolate_meshes(&a, &b, 0.5);
        let p = r.points.get(0);
        assert!((p[2] - 0.5).abs() < 1e-10);
    }
    #[test]
    fn test_seq() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [0.5, 1.0, 2.0]],
            vec![[0, 1, 2]],
        );
        let seq = interpolate_sequence(&a, &b, 4);
        assert_eq!(seq.len(), 5);
    }
}
