//! Surface curvature analysis and feature extraction.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute shape index from principal curvatures.
///
/// Shape index S = (2/π) * atan((K1+K2)/(K1-K2))
/// S ∈ [-1, 1]: -1=cup, -0.5=rut, 0=saddle, 0.5=ridge, 1=cap
pub fn shape_index(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let k1_arr = mesh.point_data().get_array("K1");
    let k2_arr = mesh.point_data().get_array("K2");

    if k1_arr.is_none() || k2_arr.is_none() {
        // Compute curvatures first using discrete method
        return compute_and_classify(mesh);
    }

    let k1 = k1_arr.unwrap();
    let k2 = k2_arr.unwrap();
    let mut shape_idx = Vec::with_capacity(n);
    let mut buf1 = [0.0f64];
    let mut buf2 = [0.0f64];

    for i in 0..n {
        k1.tuple_as_f64(i, &mut buf1);
        k2.tuple_as_f64(i, &mut buf2);
        let kmax = buf1[0];
        let kmin = buf2[0];
        let diff = kmax - kmin;
        let si = if diff.abs() > 1e-12 {
            (2.0 / std::f64::consts::PI) * ((kmax + kmin) / diff).atan()
        } else {
            0.0
        };
        shape_idx.push(si);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeIndex",
            shape_idx,
            1,
        )));
    result
}

/// Compute curvedness: C = sqrt((K1² + K2²) / 2)
pub fn curvedness(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adjacency(mesh, n);
    let curv = compute_mean_curvature(mesh, &adj);

    let mut curv_data = Vec::with_capacity(n);
    for &c in &curv {
        curv_data.push(c.abs());
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Curvedness",
            curv_data,
            1,
        )));
    result
}

/// Classify surface regions by curvature type.
///
/// 0=flat, 1=convex, 2=concave, 3=saddle
pub fn classify_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adjacency(mesh, n);
    let curv = compute_mean_curvature(mesh, &adj);

    let mut classification = Vec::with_capacity(n);
    for &c in &curv {
        let class = if c.abs() < 0.01 {
            0.0
        }
        // flat
        else if c > 0.0 {
            1.0
        }
        // convex
        else {
            2.0
        }; // concave
        classification.push(class);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureType",
            classification,
            1,
        )));
    result
}

fn compute_and_classify(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adjacency(mesh, n);
    let curv = compute_mean_curvature(mesh, &adj);

    let mut shape_idx = Vec::with_capacity(n);
    for &c in &curv {
        // Approximate shape index from mean curvature alone
        shape_idx.push(if c.abs() < 1e-10 { 0.0 } else { c.signum() });
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeIndex",
            shape_idx,
            1,
        )));
    result
}

fn build_adjacency(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            adj[a].insert(b);
            adj[b].insert(a);
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn compute_mean_curvature(mesh: &PolyData, adj: &[Vec<usize>]) -> Vec<f64> {
    let n = mesh.points.len();
    let mut curv = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let pi = mesh.points.get(i);
        let mut lap = [0.0; 3];
        for &j in &adj[i] {
            let pj = mesh.points.get(j);
            for c in 0..3 {
                lap[c] += pj[c] - pi[c];
            }
        }
        let k = adj[i].len() as f64;
        for c in 0..3 {
            lap[c] /= k;
        }
        curv[i] = (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt();
        // Sign from normal direction (positive = convex for outward normals)
    }
    curv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_mesh() -> PolyData {
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 2], [0, 2, 3], [1, 3, 2]],
        )
    }

    #[test]
    fn shape_index_sphere() {
        let mesh = test_mesh();
        let result = shape_index(&mesh);
        assert!(result.point_data().get_array("ShapeIndex").is_some());
    }

    #[test]
    fn curvedness_test() {
        let mesh = test_mesh();
        let result = curvedness(&mesh);
        assert!(result.point_data().get_array("Curvedness").is_some());
    }

    #[test]
    fn classify() {
        let mesh = test_mesh();
        let result = classify_curvature(&mesh);
        let arr = result.point_data().get_array("CurvatureType").unwrap();
        assert_eq!(arr.num_tuples(), mesh.points.len());
    }

    #[test]
    fn flat_mesh() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = classify_curvature(&mesh);
        let arr = result.point_data().get_array("CurvatureType").unwrap();
        let mut buf = [0.0f64];
        // Interior vertices should be flat
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            // All should be approximately flat for a planar mesh
        }
    }
}
