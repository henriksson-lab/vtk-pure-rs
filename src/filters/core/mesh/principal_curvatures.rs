use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Estimate principal curvatures (k1, k2) at each vertex from mean and
/// Gaussian curvature, following vtkCurvatures' triangle-mesh formulas.
///
/// Adds "PrincipalCurvature1" and "PrincipalCurvature2" point data arrays.
/// Input should be a triangle mesh.
pub fn compute_principal_curvatures(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut gauss = vec![0.0f64; n];
    let mut area = vec![0.0f64; n];

    // Initialize Gaussian curvature with 2*pi (angle deficit starts from full circle)
    gauss.fill(2.0 * std::f64::consts::PI);

    let mut triangles = Vec::new();
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let i0 = cell[0] as usize;
        let i1 = cell[1] as usize;
        let i2 = cell[2] as usize;

        let p0 = input.points.get(i0);
        let p1 = input.points.get(i1);
        let p2 = input.points.get(i2);

        let e01 = sub(p1, p0);
        let e02 = sub(p2, p0);
        let e12 = sub(p2, p1);
        let e10 = sub(p0, p1);
        let e20 = sub(p0, p2);
        let e21 = sub(p1, p2);

        let a0 = angle_between(e01, e02);
        let a1 = angle_between(e10, e12);
        let a2 = angle_between(e20, e21);

        gauss[i0] -= a0;
        gauss[i1] -= a1;
        gauss[i2] -= a2;

        let cross = cross_3(e01, e02);
        let tri_area: f64 = 0.5 * length(cross);

        area[i0] += tri_area;
        area[i1] += tri_area;
        area[i2] += tri_area;

        let tri_id = triangles.len();
        triangles.push(TriangleInfo {
            ids: [i0, i1, i2],
            normal: normalize(cross),
            area: tri_area,
        });

        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(tri_id);
        }
    }

    let mean = compute_vtk_mean_curvature(input, &triangles, &edge_faces);

    // Normalize by area and compute principal curvatures
    let mut k1 = vec![0.0f64; n];
    let mut k2 = vec![0.0f64; n];

    for i in 0..n {
        if area[i] > 1e-20 {
            let g: f64 = 3.0 * gauss[i] / area[i];
            let h: f64 = mean[i];

            let disc: f64 = (h * h - g).max(0.0);
            let sqrt_disc: f64 = disc.sqrt();
            k1[i] = h + sqrt_disc;
            k2[i] = h - sqrt_disc;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(
        DataArray::from_vec("PrincipalCurvature1", k1, 1),
    ));
    pd.point_data_mut().add_array(AnyDataArray::F64(
        DataArray::from_vec("PrincipalCurvature2", k2, 1),
    ));
    pd
}

#[derive(Clone, Copy)]
struct TriangleInfo {
    ids: [usize; 3],
    normal: [f64; 3],
    area: f64,
}

fn compute_vtk_mean_curvature(
    input: &PolyData,
    triangles: &[TriangleInfo],
    edge_faces: &HashMap<(usize, usize), Vec<usize>>,
) -> Vec<f64> {
    let mut sum = vec![0.0f64; input.points.len()];
    let mut num_neighbors = vec![0usize; input.points.len()];

    for (f, tri) in triangles.iter().enumerate() {
        for v in 0..3 {
            let v_l = tri.ids[v];
            let v_r = tri.ids[(v + 1) % 3];
            let key = if v_l < v_r { (v_l, v_r) } else { (v_r, v_l) };
            let Some(faces) = edge_faces.get(&key) else {
                continue;
            };
            if faces.len() != 2 {
                continue;
            }
            let n = if faces[0] == f { faces[1] } else { faces[0] };
            if n <= f {
                continue;
            }

            let ore = input.points.get(v_l);
            let end = input.points.get(v_r);
            let edge = sub(end, ore);
            let length = length(edge);
            if length <= 1e-20 {
                continue;
            }
            let edge_unit = [edge[0] / length, edge[1] / length, edge[2] / length];
            let neighbor = triangles[n];
            let area = tri.area + neighbor.area;

            let cs = dot(tri.normal, neighbor.normal);
            let sn = dot(cross_3(tri.normal, neighbor.normal), edge_unit);
            let hf = if sn != 0.0 || cs != 0.0 {
                length * sn.atan2(cs)
            } else {
                0.0
            };
            let hf = if area != 0.0 { 3.0 * hf / area } else { hf };

            sum[v_l] += hf;
            sum[v_r] += hf;
            num_neighbors[v_l] += 1;
            num_neighbors[v_r] += 1;
        }
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
    let la: f64 = length(a);
    let lb: f64 = length(b);
    if la < 1e-20 || lb < 1e-20 {
        return 0.0;
    }
    let cos_angle: f64 = (dot(a, b) / (la * lb)).clamp(-1.0, 1.0);
    cos_angle.acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrays_present_and_sized() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_principal_curvatures(&pd);
        let k1 = result.point_data().get_array("PrincipalCurvature1").unwrap();
        let k2 = result.point_data().get_array("PrincipalCurvature2").unwrap();
        assert_eq!(k1.num_tuples(), 3);
        assert_eq!(k2.num_tuples(), 3);
    }

    #[test]
    fn flat_mesh_k1_ge_k2() {
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
        let result = compute_principal_curvatures(&pd);
        let k1_arr = result.point_data().get_array("PrincipalCurvature1").unwrap();
        let k2_arr = result.point_data().get_array("PrincipalCurvature2").unwrap();
        let mut v1 = [0.0f64];
        let mut v2 = [0.0f64];
        for i in 0..6 {
            k1_arr.tuple_as_f64(i, &mut v1);
            k2_arr.tuple_as_f64(i, &mut v2);
            assert!(v1[0] >= v2[0] - 1e-10, "k1 should be >= k2 at vertex {}", i);
        }
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::from_triangles(vec![], vec![]);
        let result = compute_principal_curvatures(&pd);
        let k1 = result.point_data().get_array("PrincipalCurvature1").unwrap();
        assert_eq!(k1.num_tuples(), 0);
    }
}
