//! Map mesh vertices to unit sphere (spherical parameterization).
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Project mesh vertices onto unit sphere centered at centroid.
pub fn map_to_sphere(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
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
    let mut result = mesh.clone();
    let mut uvs = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        let r = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-15);
        result.points.set(i, [dx / r, dy / r, dz / r]);
        let u = (dy.atan2(dx) / (2.0 * std::f64::consts::PI) + 0.5).rem_euclid(1.0);
        let v = (dz / r).acos() / std::f64::consts::PI;
        uvs.push(u);
        uvs.push(v);
    }
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uvs, 2)));
    result
}
/// Compute sphericity (how close to a sphere: 1.0 = perfect sphere).
pub fn sphericity(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
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
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mean = dists.iter().sum::<f64>() / nf;
    if mean < 1e-15 {
        return 0.0;
    }
    let var = dists.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / nf;
    1.0 - (var.sqrt() / mean).min(1.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_map() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let r = map_to_sphere(&mesh);
        for i in 0..3 {
            let p = r.points.get(i);
            let d = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((d - 1.0).abs() < 1e-10);
        }
    }
    #[test]
    fn test_sphericity() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 2, 4],
                [0, 4, 3],
                [0, 3, 5],
                [0, 5, 2],
                [1, 4, 2],
                [1, 3, 4],
                [1, 5, 3],
                [1, 2, 5],
            ],
        );
        let s = sphericity(&mesh);
        assert!(s > 0.9); // octahedron is fairly spherical
    }
}
