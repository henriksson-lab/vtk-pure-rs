//! Convert mesh vertices to point cloud (vertices only, no cells).

use crate::data::{CellArray, Points, PolyData};

/// Extract all mesh vertices as a point cloud (PolyData with verts).
pub fn mesh_to_point_cloud(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..mesh.points.len() {
        pts.push(mesh.points.get(i));
        verts.push_cell(&[i as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    // Copy point data
    result
}

/// Create a point cloud from raw coordinate arrays.
pub fn point_cloud_from_coords(coords: &[[f64; 3]]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for (i, p) in coords.iter().enumerate() {
        pts.push(*p);
        verts.push_cell(&[i as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Compute centroid of all points.
pub fn point_cloud_centroid(mesh: &PolyData) -> [f64; 3] {
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

/// Compute bounding sphere (center + radius).
pub fn point_cloud_bounding_sphere(mesh: &PolyData) -> ([f64; 3], f64) {
    let center = point_cloud_centroid(mesh);
    let mut max_r2 = 0.0f64;
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        max_r2 = max_r2.max(d2);
    }
    (center, max_r2.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_cloud() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let pc = mesh_to_point_cloud(&mesh);
        assert_eq!(pc.points.len(), 3);
        assert_eq!(pc.verts.num_cells(), 3);
        assert_eq!(pc.polys.num_cells(), 0);
    }
    #[test]
    fn test_from_coords() {
        let pc = point_cloud_from_coords(&[[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]]);
        assert_eq!(pc.points.len(), 2);
    }
    #[test]
    fn test_centroid() {
        let pc = point_cloud_from_coords(&[[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]]);
        let c = point_cloud_centroid(&pc);
        assert!((c[0] - 1.0).abs() < 1e-10);
        assert!((c[1] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_bsphere() {
        let pc = point_cloud_from_coords(&[
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]);
        let (c, r) = point_cloud_bounding_sphere(&pc);
        assert!(c[0].abs() < 1e-10);
        assert!((r - 1.0).abs() < 1e-10);
    }
}
