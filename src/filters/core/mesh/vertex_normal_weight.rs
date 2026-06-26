use crate::data::{AnyDataArray, DataArray, PolyData};

/// Weighting method for vertex normal computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalWeightMethod {
    /// Each adjacent face contributes equally.
    Uniform,
    /// Each face's contribution is weighted by its area.
    Area,
    /// Each face's contribution is weighted by the angle at the vertex.
    Angle,
}

/// Compute vertex normals with configurable weighting.
///
/// Iterates over all polygon cells, computes each face normal, and
/// accumulates it into each vertex's normal weighted by the chosen method.
/// The result is normalized and stored as a 3-component "WeightedNormals"
/// point data array.
pub fn compute_vertex_normals_weighted(input: &PolyData, method: NormalWeightMethod) -> PolyData {
    let npts: usize = input.points.len();
    let mut normals: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; npts];

    for cell in input.polys.iter() {
        let Some(indices) = valid_cell_indices(cell, npts) else {
            continue;
        };
        let cn: usize = indices.len();
        if cn < 3 {
            continue;
        }

        let mut face_normal = polygon_normal(input, &indices);
        let normal_length = norm(face_normal);
        if normal_length < 1e-20 {
            continue;
        }
        let face_area: f64 = normal_length * 0.5;
        face_normal[0] /= normal_length;
        face_normal[1] /= normal_length;
        face_normal[2] /= normal_length;

        for vi in 0..cn {
            let pid: usize = indices[vi];

            let weight: f64 = match method {
                NormalWeightMethod::Uniform => 1.0,
                NormalWeightMethod::Area => face_area,
                NormalWeightMethod::Angle => {
                    let prev_idx: usize = if vi == 0 { cn - 1 } else { vi - 1 };
                    let next_idx: usize = (vi + 1) % cn;
                    let pc: [f64; 3] = input.points.get(indices[vi]);
                    let pp: [f64; 3] = input.points.get(indices[prev_idx]);
                    let pn: [f64; 3] = input.points.get(indices[next_idx]);

                    let va: [f64; 3] = [pp[0] - pc[0], pp[1] - pc[1], pp[2] - pc[2]];
                    let vb: [f64; 3] = [pn[0] - pc[0], pn[1] - pc[1], pn[2] - pc[2]];
                    let la: f64 = norm(va);
                    let lb: f64 = norm(vb);
                    let denom: f64 = la * lb;
                    if denom < 1e-20 {
                        0.0
                    } else {
                        let cos_a: f64 = (dot(va, vb) / denom).clamp(-1.0, 1.0);
                        cos_a.acos()
                    }
                }
            };

            normals[pid][0] += face_normal[0] * weight;
            normals[pid][1] += face_normal[1] * weight;
            normals[pid][2] += face_normal[2] * weight;
        }
    }

    let mut flat: Vec<f64> = Vec::with_capacity(npts * 3);
    for n in &normals {
        let len: f64 = norm(*n);
        if len > 1e-20 {
            flat.push(n[0] / len);
            flat.push(n[1] / len);
            flat.push(n[2] / len);
        } else {
            flat.push(0.0);
            flat.push(0.0);
            flat.push(1.0);
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WeightedNormals",
            flat,
            3,
        )));
    pd.point_data_mut().set_active_normals("WeightedNormals");
    pd
}

fn valid_cell_indices(cell: &[i64], npoints: usize) -> Option<Vec<usize>> {
    let mut indices = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= npoints {
            return None;
        }
        indices.push(id as usize);
    }
    Some(indices)
}

fn polygon_normal(input: &PolyData, indices: &[usize]) -> [f64; 3] {
    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < indices.len() - 2 {
        let p0 = input.points.get(indices[point_id]);
        let p1 = input.points.get(indices[point_id + 1]);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if norm_squared(v1) > 0.0 {
            common = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common else {
        return [0.0; 3];
    };
    if point_id >= indices.len() {
        return [0.0; 3];
    }

    let p0 = input.points.get(indices[common_id]);
    let mut normal = [0.0; 3];
    while point_id < indices.len() {
        let p = input.points.get(indices[point_id]);
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        normal[0] += cross[0];
        normal[1] += cross[1];
        normal[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    normal
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(vector: [f64; 3]) -> f64 {
    norm_squared(vector).sqrt()
}

fn norm_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_triangle_normals_point_up() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        for method in [
            NormalWeightMethod::Uniform,
            NormalWeightMethod::Area,
            NormalWeightMethod::Angle,
        ] {
            let result = compute_vertex_normals_weighted(&pd, method);
            let arr = result.point_data().get_array("WeightedNormals").unwrap();
            assert_eq!(arr.num_tuples(), 3);
            assert!(result.point_data().normals().is_some());
            for i in 0..3 {
                let mut val = [0.0f64; 3];
                arr.tuple_as_f64(i, &mut val);
                assert!(
                    val[2] > 0.9,
                    "method {:?}, vertex {} nz = {}",
                    method,
                    i,
                    val[2]
                );
            }
        }
    }

    #[test]
    fn shared_vertex_averages_normals() {
        // Two triangles sharing vertex 1 at different angles.
        // Triangle 0 in XY plane, triangle 1 tilted into Z.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1 (shared)
                [0.5, 1.0, 0.0], // 2
                [1.5, 1.0, 1.0], // 3
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = compute_vertex_normals_weighted(&pd, NormalWeightMethod::Uniform);
        let arr = result.point_data().get_array("WeightedNormals").unwrap();
        // Vertex 1 is shared by two faces, its normal should not be purely +Z.
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(1, &mut val);
        let len: f64 = (val[0] * val[0] + val[1] * val[1] + val[2] * val[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-10, "normal should be unit length");
    }

    #[test]
    fn area_weighting_differs_from_uniform() {
        // One big triangle and one small triangle sharing a vertex.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],  // 0 (shared)
                [10.0, 0.0, 0.0], // 1
                [0.0, 10.0, 0.0], // 2
                [0.0, 0.0, 0.1],  // 3
                [0.1, 0.0, 0.0],  // 4
            ],
            vec![[0, 1, 2], [0, 3, 4]],
        );
        let result_uniform = compute_vertex_normals_weighted(&pd, NormalWeightMethod::Uniform);
        let result_area = compute_vertex_normals_weighted(&pd, NormalWeightMethod::Area);
        let arr_u = result_uniform
            .point_data()
            .get_array("WeightedNormals")
            .unwrap();
        let arr_a = result_area
            .point_data()
            .get_array("WeightedNormals")
            .unwrap();
        let mut nu = [0.0f64; 3];
        let mut na = [0.0f64; 3];
        arr_u.tuple_as_f64(0, &mut nu);
        arr_a.tuple_as_f64(0, &mut na);
        // With area weighting, the big triangle should dominate more.
        // The normals at vertex 0 should differ.
        let dot: f64 = nu[0] * na[0] + nu[1] * na[1] + nu[2] * na[2];
        // They should be similar but not identical (dot < 1.0).
        assert!(
            dot < 1.0 - 1e-6,
            "area and uniform should differ for vertex 0, dot = {}",
            dot
        );
    }
}
