//! Detect reflective symmetry planes in a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn symmetry_score(mesh: &PolyData, plane_normal: [f64; 3], plane_point: [f64; 3]) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
    }
    let nl = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt();
    if nl < 1e-15 {
        return 0.0;
    }
    let nn = [
        plane_normal[0] / nl,
        plane_normal[1] / nl,
        plane_normal[2] / nl,
    ];
    // Reflect each point and find nearest original point
    let mut total_dist = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = (p[0] - plane_point[0]) * nn[0]
            + (p[1] - plane_point[1]) * nn[1]
            + (p[2] - plane_point[2]) * nn[2];
        let reflected = [
            p[0] - 2.0 * d * nn[0],
            p[1] - 2.0 * d * nn[1],
            p[2] - 2.0 * d * nn[2],
        ];
        let mut min_dist = f64::INFINITY;
        for j in 0..n {
            let q = mesh.points.get(j);
            let dist = ((reflected[0] - q[0]).powi(2)
                + (reflected[1] - q[1]).powi(2)
                + (reflected[2] - q[2]).powi(2))
            .sqrt();
            if dist < min_dist {
                min_dist = dist;
            }
        }
        total_dist += min_dist;
    }
    total_dist / n as f64
}

pub fn detect_symmetry_planes(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Compute centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    // Test principal planes
    let planes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let scores: Vec<f64> = planes
        .iter()
        .map(|&nn| symmetry_score(mesh, nn, [cx, cy, cz]))
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SymmetryXScore",
            vec![scores[0]; n],
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SymmetryYScore",
            vec![scores[1]; n],
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SymmetryZScore",
            vec![scores[2]; n],
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_symmetry() {
        let mesh = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let score = symmetry_score(&mesh, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(score < 0.01); // symmetric about YZ plane
    }
}
