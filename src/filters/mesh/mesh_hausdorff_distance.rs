//! Compute one-sided and symmetric Hausdorff distance between two meshes.
use crate::data::PolyData;

pub fn hausdorff_distance(mesh_a: &PolyData, mesh_b: &PolyData) -> (f64, f64, f64) {
    let na = mesh_a.points.len();
    let nb = mesh_b.points.len();
    if na == 0 || nb == 0 {
        return (0.0, 0.0, 0.0);
    }
    // One-sided: max over A of min distance to B
    let d_ab = (0..na)
        .map(|i| {
            let pa = mesh_a.points.get(i);
            (0..nb)
                .map(|j| {
                    let pb = mesh_b.points.get(j);
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt()
                })
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0f64, f64::max);
    let d_ba = (0..nb)
        .map(|i| {
            let pb = mesh_b.points.get(i);
            (0..na)
                .map(|j| {
                    let pa = mesh_a.points.get(j);
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt()
                })
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0f64, f64::max);
    let symmetric = d_ab.max(d_ba);
    (d_ab, d_ba, symmetric)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hausdorff() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let (dab, dba, sym) = hausdorff_distance(&a, &b);
        assert!((dab - 1.0).abs() < 0.01);
        assert!((dba - 1.0).abs() < 0.01);
        assert!((sym - 1.0).abs() < 0.01);
    }
}
