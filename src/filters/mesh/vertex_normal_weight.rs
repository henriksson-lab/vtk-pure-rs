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
        let cn: usize = cell.len();
        if cn < 3 {
            continue;
        }

        // Compute face normal and area via cross product of first two edges.
        let p0: [f64; 3] = input.points.get(cell[0] as usize);
        let p1: [f64; 3] = input.points.get(cell[1] as usize);
        let p2: [f64; 3] = input.points.get(cell[2] as usize);

        let e1: [f64; 3] = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2: [f64; 3] = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cross: [f64; 3] = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let cross_len: f64 =
            (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if cross_len < 1e-20 {
            continue;
        }
        let face_normal: [f64; 3] = [
            cross[0] / cross_len,
            cross[1] / cross_len,
            cross[2] / cross_len,
        ];
        let face_area: f64 = cross_len * 0.5;

        for vi in 0..cn {
            let pid: usize = cell[vi] as usize;

            let weight: f64 = match method {
                NormalWeightMethod::Uniform => 1.0,
                NormalWeightMethod::Area => face_area,
                NormalWeightMethod::Angle => {
                    // Angle at vertex vi in this polygon.
                    let prev_idx: usize = if vi == 0 { cn - 1 } else { vi - 1 };
                    let next_idx: usize = (vi + 1) % cn;
                    let pc: [f64; 3] = input.points.get(cell[vi] as usize);
                    let pp: [f64; 3] = input.points.get(cell[prev_idx] as usize);
                    let pn: [f64; 3] = input.points.get(cell[next_idx] as usize);

                    let va: [f64; 3] = [pp[0] - pc[0], pp[1] - pc[1], pp[2] - pc[2]];
                    let vb: [f64; 3] = [pn[0] - pc[0], pn[1] - pc[1], pn[2] - pc[2]];
                    let dot: f64 = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
                    let la: f64 = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
                    let lb: f64 = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
                    let denom: f64 = la * lb;
                    if denom < 1e-20 {
                        0.0
                    } else {
                        let cos_a: f64 = (dot / denom).clamp(-1.0, 1.0);
                        cos_a.acos()
                    }
                }
            };

            normals[pid][0] += face_normal[0] * weight;
            normals[pid][1] += face_normal[1] * weight;
            normals[pid][2] += face_normal[2] * weight;
        }
    }

    // Normalize.
    let mut flat: Vec<f64> = Vec::with_capacity(npts * 3);
    for n in &normals {
        let len: f64 = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
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
    pd
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
