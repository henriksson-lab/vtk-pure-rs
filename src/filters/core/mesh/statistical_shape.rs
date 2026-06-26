//! Statistical shape analysis: mean shape, shape distance, PCA on shapes.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Compute the mean shape from a collection of meshes with corresponding vertices.
///
/// All meshes must have the same number of vertices in correspondence.
pub fn mean_shape(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    let n = meshes[0].points.len();
    if n == 0 {
        return meshes[0].clone();
    }
    assert!(
        meshes.iter().all(|mesh| mesh.points.len() == n),
        "all shapes must have the same number of points"
    );

    let mut avg = vec![[0.0; 3]; n];
    for mesh in meshes {
        for i in 0..n {
            let p = mesh.points.get(i);
            for c in 0..3 {
                avg[i][c] += p[c];
            }
        }
    }
    let k = meshes.len() as f64;
    for i in 0..n {
        for c in 0..3 {
            avg[i][c] /= k;
        }
    }

    let mut result = meshes[0].clone();
    result.points = Points::from(avg);
    result
}

/// Compute per-vertex variance across a shape collection.
pub fn shape_variance(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    let n = meshes[0].points.len();
    assert!(
        meshes.iter().all(|mesh| mesh.points.len() == n),
        "all shapes must have the same number of points"
    );
    let mean = mean_shape(meshes);

    let mut variance = vec![0.0f64; n];
    for mesh in meshes {
        for i in 0..n {
            let p = mesh.points.get(i);
            let m = mean.points.get(i);
            variance[i] += (p[0] - m[0]).powi(2) + (p[1] - m[1]).powi(2) + (p[2] - m[2]).powi(2);
        }
    }
    let k = meshes.len() as f64;
    for v in &mut variance {
        *v = (*v / k).sqrt();
    }

    let mut result = mean;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeVariance",
            variance,
            1,
        )));
    result
}

/// Compute Procrustes distance between two shapes.
///
/// Measures shape difference after optimal alignment (translation + uniform scale).
pub fn procrustes_distance(a: &PolyData, b: &PolyData) -> f64 {
    assert_eq!(
        a.points.len(),
        b.points.len(),
        "shapes must have the same number of points"
    );
    let n = a.points.len();
    if n == 0 {
        return 0.0;
    }

    // Compute centroids
    let mut ca = [0.0; 3];
    let mut cb = [0.0; 3];
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        for c in 0..3 {
            ca[c] += pa[c];
            cb[c] += pb[c];
        }
    }
    for c in 0..3 {
        ca[c] /= n as f64;
        cb[c] /= n as f64;
    }

    // Compute scale
    let mut scale_a = 0.0;
    let mut scale_b = 0.0;
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        scale_a += (pa[0] - ca[0]).powi(2) + (pa[1] - ca[1]).powi(2) + (pa[2] - ca[2]).powi(2);
        scale_b += (pb[0] - cb[0]).powi(2) + (pb[1] - cb[1]).powi(2) + (pb[2] - cb[2]).powi(2);
    }
    let sa = scale_a.sqrt().max(1e-15);
    let sb = scale_b.sqrt().max(1e-15);

    // Compute distance after centering and scaling
    let mut dist = 0.0;
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        for c in 0..3 {
            let va = (pa[c] - ca[c]) / sa;
            let vb = (pb[c] - cb[c]) / sb;
            dist += (va - vb).powi(2);
        }
    }
    (dist / n as f64).sqrt()
}

/// Compute displacement vectors between two shapes.
pub fn shape_displacement(from: &PolyData, to: &PolyData) -> PolyData {
    assert_eq!(
        from.points.len(),
        to.points.len(),
        "shapes must have the same number of points"
    );
    let n = from.points.len();
    let mut disp = Vec::with_capacity(n * 3);
    let mut mag = Vec::with_capacity(n);
    for i in 0..n {
        let a = from.points.get(i);
        let b = to.points.get(i);
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        disp.extend_from_slice(&d);
        mag.push((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    let mut result = from.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Displacement",
            disp,
            3,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DisplacementMag",
            mag,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mean() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
        let m = mean_shape(&[&a, &b]);
        assert!((m.points.get(1)[0] - 3.0).abs() < 0.01);
    }
    #[test]
    fn variance() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        let result = shape_variance(&[&a, &b]);
        assert!(result.point_data().get_array("ShapeVariance").is_some());
    }
    #[test]
    fn procrustes() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]); // scaled
        let d = procrustes_distance(&a, &b);
        assert!(
            d < 0.01,
            "same shape scaled should have small Procrustes distance, got {d}"
        );
    }
    #[test]
    fn displacement() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 2.0, 3.0]]);
        let result = shape_displacement(&a, &b);
        assert!(result.point_data().get_array("Displacement").is_some());
        assert!(result.point_data().get_array("DisplacementMag").is_some());
    }
}
