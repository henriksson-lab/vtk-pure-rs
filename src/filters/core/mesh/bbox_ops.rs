//! Bounding box operations: compute, expand, test containment.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute axis-aligned bounding box as [min_x, max_x, min_y, max_y, min_z, max_z].
pub fn compute_aabb(mesh: &PolyData) -> [f64; 6] {
    let n = mesh.points.len();
    if n == 0 {
        return [0.0; 6];
    }
    let mut min = mesh.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }
    [min[0], max[0], min[1], max[1], min[2], max[2]]
}

/// Compute bounding box dimensions [width, height, depth].
pub fn aabb_dimensions(mesh: &PolyData) -> [f64; 3] {
    let bb = compute_aabb(mesh);
    [bb[1] - bb[0], bb[3] - bb[2], bb[5] - bb[4]]
}

/// Compute bounding box volume.
pub fn aabb_volume(mesh: &PolyData) -> f64 {
    let d = aabb_dimensions(mesh);
    d[0] * d[1] * d[2]
}

/// Compute bounding box surface area.
pub fn aabb_surface_area(mesh: &PolyData) -> f64 {
    let d = aabb_dimensions(mesh);
    2.0 * (d[0] * d[1] + d[1] * d[2] + d[0] * d[2])
}

/// Compute bounding box diagonal length.
pub fn aabb_diagonal(mesh: &PolyData) -> f64 {
    let d = aabb_dimensions(mesh);
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Test if a point is inside the bounding box (with optional padding).
pub fn point_in_aabb(mesh: &PolyData, point: [f64; 3], padding: f64) -> bool {
    let bb = compute_aabb(mesh);
    point[0] >= bb[0] - padding
        && point[0] <= bb[1] + padding
        && point[1] >= bb[2] - padding
        && point[1] <= bb[3] + padding
        && point[2] >= bb[4] - padding
        && point[2] <= bb[5] + padding
}

/// Add per-vertex normalized position within bounding box as point data.
pub fn normalized_bbox_coordinates(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let bb = compute_aabb(mesh);
    let rx = (bb[1] - bb[0]).max(1e-15);
    let ry = (bb[3] - bb[2]).max(1e-15);
    let rz = (bb[5] - bb[4]).max(1e-15);
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = mesh.points.get(i);
        data.push((p[0] - bb[0]) / rx);
        data.push((p[1] - bb[2]) / ry);
        data.push((p[2] - bb[4]) / rz);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalizedCoords",
            data,
            3,
        )));
    result
}

/// Bounding box info as formatted string.
pub fn aabb_info_string(mesh: &PolyData) -> String {
    let bb = compute_aabb(mesh);
    let d = aabb_dimensions(mesh);
    format!("AABB: [{:.3},{:.3}]×[{:.3},{:.3}]×[{:.3},{:.3}], size=[{:.3},{:.3},{:.3}], diag={:.3}, vol={:.3}",
        bb[0],bb[1],bb[2],bb[3],bb[4],bb[5],d[0],d[1],d[2],aabb_diagonal(mesh),aabb_volume(mesh))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aabb() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [3.0, 4.0, 5.0]]);
        let bb = compute_aabb(&mesh);
        assert_eq!(bb[0], 0.0);
        assert_eq!(bb[1], 3.0);
        assert_eq!(aabb_dimensions(&mesh), [3.0, 4.0, 5.0]);
        assert_eq!(aabb_volume(&mesh), 60.0);
    }
    #[test]
    fn contains() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [10.0, 10.0, 10.0]]);
        assert!(point_in_aabb(&mesh, [5.0, 5.0, 5.0], 0.0));
        assert!(!point_in_aabb(&mesh, [15.0, 5.0, 5.0], 0.0));
    }
    #[test]
    fn normalized() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [10.0, 20.0, 30.0]]);
        let result = normalized_bbox_coordinates(&mesh);
        let arr = result.point_data().get_array("NormalizedCoords").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }
    #[test]
    fn info() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        let s = aabb_info_string(&mesh);
        assert!(s.contains("AABB"));
    }
}
