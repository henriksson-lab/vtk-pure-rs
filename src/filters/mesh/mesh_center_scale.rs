//! Normalize mesh to unit bounding box or unit sphere.

use crate::data::PolyData;

/// Normalize mesh to fit in [-1, 1]^3 centered at origin.
pub fn normalize_to_unit_box(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let (mn, mx) = bounds(mesh);
    let cx = (mn[0] + mx[0]) / 2.0;
    let cy = (mn[1] + mx[1]) / 2.0;
    let cz = (mn[2] + mx[2]) / 2.0;
    let scale = [
        (mx[0] - mn[0]) / 2.0,
        (mx[1] - mn[1]) / 2.0,
        (mx[2] - mn[2]) / 2.0,
    ];
    let max_scale = scale[0].max(scale[1]).max(scale[2]).max(1e-15);

    let mut result = mesh.clone();
    for i in 0..n {
        let p = result.points.get(i);
        result.points.set(
            i,
            [
                (p[0] - cx) / max_scale,
                (p[1] - cy) / max_scale,
                (p[2] - cz) / max_scale,
            ],
        );
    }
    result
}

/// Normalize mesh to fit in a unit sphere centered at origin.
pub fn normalize_to_unit_sphere(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let (mn, mx) = bounds(mesh);
    let cx = (mn[0] + mx[0]) / 2.0;
    let cy = (mn[1] + mx[1]) / 2.0;
    let cz = (mn[2] + mx[2]) / 2.0;

    // Find max distance from center
    let mut max_r = 0.0f64;
    for i in 0..n {
        let p = mesh.points.get(i);
        let r = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt();
        max_r = max_r.max(r);
    }
    if max_r < 1e-15 {
        max_r = 1.0;
    }

    let mut result = mesh.clone();
    for i in 0..n {
        let p = result.points.get(i);
        result.points.set(
            i,
            [
                (p[0] - cx) / max_r,
                (p[1] - cy) / max_r,
                (p[2] - cz) / max_r,
            ],
        );
    }
    result
}

/// Scale mesh to fit in a box of given size, centered at origin.
pub fn fit_to_size(mesh: &PolyData, size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let normalized = normalize_to_unit_box(mesh);
    let half = size / 2.0;
    let mut result = normalized;
    for i in 0..n {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0] * half, p[1] * half, p[2] * half]);
    }
    result
}

fn bounds(mesh: &PolyData) -> ([f64; 3], [f64; 3]) {
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unit_box() {
        let mesh = PolyData::from_triangles(
            vec![[10.0, 20.0, 30.0], [12.0, 20.0, 30.0], [11.0, 22.0, 30.0]],
            vec![[0, 1, 2]],
        );
        let r = normalize_to_unit_box(&mesh);
        for i in 0..3 {
            let p = r.points.get(i);
            assert!(p[0] >= -1.01 && p[0] <= 1.01);
            assert!(p[1] >= -1.01 && p[1] <= 1.01);
        }
    }
    #[test]
    fn test_unit_sphere() {
        let mesh = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [-5.0, 0.0, 0.0], [0.0, 5.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = normalize_to_unit_sphere(&mesh);
        for i in 0..3 {
            let p = r.points.get(i);
            let d = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(d <= 1.01);
        }
    }
    #[test]
    fn test_fit() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = fit_to_size(&mesh, 4.0);
        let (mn, mx) = bounds(&r);
        assert!((mx[0] - mn[0]) <= 4.01);
    }
}
