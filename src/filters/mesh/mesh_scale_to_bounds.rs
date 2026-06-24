//! Scale mesh to fit within specific bounds.
use crate::data::PolyData;
pub fn scale_to_bounds(mesh: &PolyData, target_min: [f64; 3], target_max: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let mut np = [0.0; 3];
        for j in 0..3 {
            let range = (mx[j] - mn[j]).max(1e-15);
            np[j] = target_min[j] + (p[j] - mn[j]) / range * (target_max[j] - target_min[j]);
        }
        r.points.set(i, np);
    }
    r
}
pub fn scale_to_unit_cube(mesh: &PolyData) -> PolyData {
    scale_to_bounds(mesh, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
}
pub fn scale_to_centered_cube(mesh: &PolyData, half_size: f64) -> PolyData {
    scale_to_bounds(
        mesh,
        [-half_size, -half_size, -half_size],
        [half_size, half_size, half_size],
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bounds() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = scale_to_bounds(&m, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        for i in 0..3 {
            let p = r.points.get(i);
            for j in 0..3 {
                assert!(p[j] >= -0.01 && p[j] <= 1.01);
            }
        }
    }
    #[test]
    fn test_unit() {
        let m = PolyData::from_triangles(
            vec![[5.0, 10.0, 15.0], [8.0, 12.0, 18.0], [6.0, 11.0, 16.0]],
            vec![[0, 1, 2]],
        );
        let r = scale_to_unit_cube(&m);
        let p = r.points.get(0);
        assert!(p[0] >= 0.0 && p[0] <= 1.0);
    }
}
