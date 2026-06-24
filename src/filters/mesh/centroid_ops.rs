//! Centroid-based operations: per-cell centroid, centroid-distance, centroid cloud.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Compute per-cell centroid as a point cloud.
pub fn cell_centroids_as_points(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut area = Vec::new();
    for cell in mesh.polys.iter() {
        let mut c = [0.0; 3];
        for &pid in cell {
            let p = mesh.points.get(pid as usize);
            for j in 0..3 {
                c[j] += p[j];
            }
        }
        let k = cell.len() as f64;
        pts.push([c[0] / k, c[1] / k, c[2] / k]);
        if cell.len() >= 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let cc = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
            area.push(
                0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt(),
            );
        } else {
            area.push(0.0);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CellArea", area, 1)));
    result
}

/// Add distance from each vertex to the mesh centroid as point data.
pub fn distance_from_centroid(mesh: &PolyData) -> PolyData {
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
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CentroidDistance",
            data,
            1,
        )));
    result
}

/// Center mesh at origin (translate so centroid is at [0,0,0]).
pub fn center_at_origin(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] - c[0], p[1] - c[1], p[2] - c[2]]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

/// Normalize mesh to fit within a unit sphere centered at origin.
pub fn normalize_to_unit_sphere(mesh: &PolyData) -> PolyData {
    let centered = center_at_origin(mesh);
    let n = centered.points.len();
    if n == 0 {
        return centered;
    }
    let mut max_r = 0.0f64;
    for i in 0..n {
        let p = centered.points.get(i);
        max_r = max_r.max((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt());
    }
    if max_r < 1e-15 {
        return centered;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = centered.points.get(i);
        pts.push([p[0] / max_r, p[1] / max_r, p[2] / max_r]);
    }
    let mut result = centered;
    result.points = pts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn centroids() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = cell_centroids_as_points(&mesh);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 0.01);
    }
    #[test]
    fn dist() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = distance_from_centroid(&mesh);
        let arr = result.point_data().get_array("CentroidDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // distance from centroid (1,0,0) to (0,0,0)
    }
    #[test]
    fn center() {
        let mesh = PolyData::from_points(vec![[2.0, 4.0, 6.0], [4.0, 6.0, 8.0]]);
        let result = center_at_origin(&mesh);
        let p = result.points.get(0);
        assert!((p[0] + 1.0).abs() < 0.01);
    }
    #[test]
    fn normalize() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
        let result = normalize_to_unit_sphere(&mesh);
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() <= 1.01);
        }
    }
}
