use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian and mean curvatures at each vertex.
///
/// Uses the angle-deficit method for Gaussian curvature and the
/// cotangent-weight formula for mean curvature. Only works on
/// triangle meshes.
pub fn curvatures(input: &PolyData) -> PolyData {
    let gauss = compute_gaussian_curvature(input);
    let mean = compute_mean_curvature(input);

    let mut pd = input.clone();
    add_gaussian_arrays(&mut pd, gauss);
    add_mean_arrays(&mut pd, mean);
    pd.point_data_mut().set_active_scalars("Mean_Curvature");
    pd
}

/// Compute only Gaussian curvature, matching `vtkCurvatures::SetCurvatureTypeToGaussian`.
pub fn curvatures_gaussian(input: &PolyData) -> PolyData {
    let gauss = compute_gaussian_curvature(input);
    let mut pd = input.clone();
    add_gaussian_arrays(&mut pd, gauss);
    pd.point_data_mut().set_active_scalars("Gauss_Curvature");
    pd
}

/// Compute only mean curvature, matching `vtkCurvatures::SetCurvatureTypeToMean`.
pub fn curvatures_mean(input: &PolyData) -> PolyData {
    let mean = compute_mean_curvature(input);
    let mut pd = input.clone();
    add_mean_arrays(&mut pd, mean);
    pd.point_data_mut().set_active_scalars("Mean_Curvature");
    pd
}

fn compute_gaussian_curvature(input: &PolyData) -> Vec<f64> {
    let n = input.points.len();
    let mut gauss = vec![0.0f64; n];
    let mut area = vec![0.0f64; n];

    // Initialize Gaussian curvature with 2*pi (angle deficit starts from full circle)
    gauss.fill(2.0 * std::f64::consts::PI);

    // Use raw offsets/connectivity and flat point access for speed.
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();
    let nc = input.polys.num_cells();
    let pts = input.points.as_flat_slice();

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        if end - start != 3 {
            continue;
        }
        let i0 = conn[start] as usize;
        let i1 = conn[start + 1] as usize;
        let i2 = conn[start + 2] as usize;

        let b0 = i0 * 3;
        let b1 = i1 * 3;
        let b2 = i2 * 3;
        let p0 = [pts[b0], pts[b0 + 1], pts[b0 + 2]];
        let p1 = [pts[b1], pts[b1 + 1], pts[b1 + 2]];
        let p2 = [pts[b2], pts[b2 + 1], pts[b2 + 2]];

        let e01 = sub(p1, p0);
        let e12 = sub(p2, p1);
        let e20 = sub(p0, p2);

        // Match vtkCurvatures::ComputeGaussCurvature: alpha0 is the
        // exterior angle opposite vertex 0, and K uses alpha1/2/0.
        let alpha0 = std::f64::consts::PI - angle_between(e12, e20);
        let alpha1 = std::f64::consts::PI - angle_between(e20, e01);
        let alpha2 = std::f64::consts::PI - angle_between(e01, e12);

        gauss[i0] -= alpha1;
        gauss[i1] -= alpha2;
        gauss[i2] -= alpha0;

        // Triangle area
        let cross = cross_3(e01, sub(p2, p0));
        let tri_area = 0.5 * length(cross);

        area[i0] += tri_area;
        area[i1] += tri_area;
        area[i2] += tri_area;
    }

    for i in 0..n {
        if area[i] > 0.0 {
            gauss[i] = 3.0 * gauss[i] / area[i];
        } else {
            gauss[i] = 0.0;
        }
    }

    gauss
}

fn add_gaussian_arrays(pd: &mut PolyData, gauss: Vec<f64>) {
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Gauss_Curvature",
            gauss.clone(),
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussCurvature",
            gauss,
            1,
        )));
}

fn add_mean_arrays(pd: &mut PolyData, mean: Vec<f64>) {
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Mean_Curvature",
            mean.clone(),
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            mean,
            1,
        )));
}

#[derive(Clone, Copy)]
struct TriangleInfo {
    normal: [f64; 3],
    area: f64,
}

fn compute_mean_curvature(input: &PolyData) -> Vec<f64> {
    let (triangles, mut edges) = build_triangle_topology(input);
    let mut sum = vec![0.0f64; input.points.len()];
    let mut num_neighbors = vec![0usize; input.points.len()];

    edges.sort_unstable_by_key(|edge| (edge.lo, edge.hi));
    let mut edge_idx = 0usize;
    while edge_idx < edges.len() {
        let lo = edges[edge_idx].lo;
        let hi = edges[edge_idx].hi;
        let start = edge_idx;
        edge_idx += 1;
        while edge_idx < edges.len() && edges[edge_idx].lo == lo && edges[edge_idx].hi == hi {
            edge_idx += 1;
        }
        if edge_idx - start != 2 {
            continue;
        }

        let first = edges[start];
        let second = edges[start + 1];
        let (edge, neighbor_id) = if first.tri < second.tri {
            (first, second.tri)
        } else {
            (second, first.tri)
        };

        let tri = triangles[edge.tri];
        let ore = input.points.get(edge.a);
        let end = input.points.get(edge.b);
        let edge_vec = sub(end, ore);
        let length = length(edge_vec);
        if length <= 1e-20 {
            continue;
        }
        let edge_unit = [
            edge_vec[0] / length,
            edge_vec[1] / length,
            edge_vec[2] / length,
        ];
        let neighbor = triangles[neighbor_id];
        let area = tri.area + neighbor.area;

        let cs = dot(tri.normal, neighbor.normal);
        let sn = dot(cross_3(tri.normal, neighbor.normal), edge_unit);
        let hf = if sn != 0.0 || cs != 0.0 {
            length * sn.atan2(cs)
        } else {
            0.0
        };
        let hf = if area != 0.0 { 3.0 * hf / area } else { hf };

        sum[edge.a] += hf;
        sum[edge.b] += hf;
        num_neighbors[edge.a] += 1;
        num_neighbors[edge.b] += 1;
    }

    sum.into_iter()
        .zip(num_neighbors)
        .map(|(value, count)| {
            if count > 0 {
                0.5 * value / count as f64
            } else {
                0.0
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
struct EdgeOccurrence {
    lo: usize,
    hi: usize,
    a: usize,
    b: usize,
    tri: usize,
}

fn build_triangle_topology(input: &PolyData) -> (Vec<TriangleInfo>, Vec<EdgeOccurrence>) {
    let offsets = input.polys.offsets();
    let conn = input.polys.connectivity();
    let nc = input.polys.num_cells();
    let pts = input.points.as_flat_slice();

    let mut triangles = Vec::with_capacity(nc);
    let mut edges = Vec::with_capacity(nc * 3);
    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        if end - start != 3 {
            continue;
        }
        let i0 = conn[start] as usize;
        let i1 = conn[start + 1] as usize;
        let i2 = conn[start + 2] as usize;

        let b0 = i0 * 3;
        let b1 = i1 * 3;
        let b2 = i2 * 3;
        let p0 = [pts[b0], pts[b0 + 1], pts[b0 + 2]];
        let p1 = [pts[b1], pts[b1 + 1], pts[b1 + 2]];
        let p2 = [pts[b2], pts[b2 + 1], pts[b2 + 2]];

        let e01 = sub(p1, p0);
        let cross = cross_3(e01, sub(p2, p0));
        let tri_area = 0.5 * length(cross);

        let tri_id = triangles.len();
        triangles.push(TriangleInfo {
            normal: normalize(cross),
            area: tri_area,
        });
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            edges.push(EdgeOccurrence {
                lo,
                hi,
                a,
                b,
                tri: tri_id,
            });
        }
    }
    (triangles, edges)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len > 1e-20 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let la = length(a);
    let lb = length(b);
    if la < 1e-20 || lb < 1e-20 {
        return 0.0;
    }
    let cos_angle = (dot(a, b) / (la * lb)).clamp(-1.0, 1.0);
    cos_angle.acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curvatures_on_flat_mesh() {
        // A flat plane — curvatures arrays should be present and have correct size
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]],
        );
        let result = curvatures(&pd);
        let gc = result.point_data().get_array("GaussCurvature").unwrap();
        let mc = result.point_data().get_array("MeanCurvature").unwrap();
        assert_eq!(gc.num_tuples(), 6);
        assert_eq!(mc.num_tuples(), 6);
    }

    #[test]
    fn curvatures_arrays_present() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = curvatures(&pd);
        assert!(result.point_data().get_array("GaussCurvature").is_some());
        assert!(result.point_data().get_array("MeanCurvature").is_some());
    }
}
