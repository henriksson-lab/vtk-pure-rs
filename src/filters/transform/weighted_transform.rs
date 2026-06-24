//! Multi-transform blending for skeletal animation.
//!
//! Each point is transformed by a weighted combination of multiple 4x4
//! transformation matrices, enabling skeletal deformation.

use crate::data::{Points, PolyData};

/// A transform with a weight for blending.
#[derive(Debug, Clone)]
pub struct WeightedTransformEntry {
    /// 4x4 transformation matrix (row-major).
    pub matrix: [f64; 16],
    /// Weight for this transform.
    pub weight: f64,
}

/// Apply weighted transform blending to mesh points.
///
/// Each point is transformed by the weighted sum of all transforms.
/// Weights are normalized to sum to 1.
pub fn weighted_transform(mesh: &PolyData, transforms: &[WeightedTransformEntry]) -> PolyData {
    if transforms.is_empty() || mesh.points.len() == 0 {
        return mesh.clone();
    }

    let total_weight: f64 = transforms.iter().map(|t| t.weight).sum();
    if total_weight < 1e-15 {
        return mesh.clone();
    }

    let n = mesh.points.len();
    let mut new_points = Points::<f64>::new();

    for i in 0..n {
        let p = mesh.points.get(i);
        let mut result = [0.0; 3];

        for t in transforms {
            let w = t.weight / total_weight;
            let m = &t.matrix;
            let tx = m[0] * p[0] + m[1] * p[1] + m[2] * p[2] + m[3];
            let ty = m[4] * p[0] + m[5] * p[1] + m[6] * p[2] + m[7];
            let tz = m[8] * p[0] + m[9] * p[1] + m[10] * p[2] + m[11];
            result[0] += w * tx;
            result[1] += w * ty;
            result[2] += w * tz;
        }

        new_points.push(result);
    }

    let mut result = mesh.clone();
    result.points = new_points;
    result
}

/// Apply per-vertex weighted transforms using bone weight arrays.
///
/// `bone_weights` has shape [n_points × n_bones] stored flat.
/// `bone_matrices` has one 4x4 matrix per bone.
pub fn skeletal_transform(
    mesh: &PolyData,
    bone_matrices: &[[f64; 16]],
    weight_array_name: &str,
) -> PolyData {
    let n = mesh.points.len();
    let n_bones = bone_matrices.len();
    if n == 0 || n_bones == 0 {
        return mesh.clone();
    }

    let weights = match mesh.point_data().get_array(weight_array_name) {
        Some(a) if a.num_components() == n_bones => a,
        _ => return mesh.clone(),
    };

    let mut new_points = Points::<f64>::new();
    let mut buf = vec![0.0f64; n_bones];

    for i in 0..n {
        let p = mesh.points.get(i);
        weights.tuple_as_f64(i, &mut buf);

        let mut result = [0.0; 3];
        let w_sum: f64 = buf.iter().sum();
        if w_sum < 1e-15 {
            new_points.push(p);
            continue;
        }

        for (bi, m) in bone_matrices.iter().enumerate() {
            let w = buf[bi] / w_sum;
            if w < 1e-15 {
                continue;
            }
            result[0] += w * (m[0] * p[0] + m[1] * p[1] + m[2] * p[2] + m[3]);
            result[1] += w * (m[4] * p[0] + m[5] * p[1] + m[6] * p[2] + m[7]);
            result[2] += w * (m[8] * p[0] + m[9] * p[1] + m[10] * p[2] + m[11]);
        }

        new_points.push(result);
    }

    let mut result = mesh.clone();
    result.points = new_points;
    result
}

/// Create an identity 4x4 matrix.
pub fn identity_matrix() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Create a translation 4x4 matrix.
pub fn translation_matrix(tx: f64, ty: f64, tz: f64) -> [f64; 16] {
    [
        1.0, 0.0, 0.0, tx, 0.0, 1.0, 0.0, ty, 0.0, 0.0, 1.0, tz, 0.0, 0.0, 0.0, 1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_transform() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let result = weighted_transform(
            &mesh,
            &[WeightedTransformEntry {
                matrix: identity_matrix(),
                weight: 1.0,
            }],
        );
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn translation_blend() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let result = weighted_transform(
            &mesh,
            &[
                WeightedTransformEntry {
                    matrix: translation_matrix(2.0, 0.0, 0.0),
                    weight: 0.5,
                },
                WeightedTransformEntry {
                    matrix: translation_matrix(0.0, 4.0, 0.0),
                    weight: 0.5,
                },
            ],
        );
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10); // 50% of 2.0
        assert!((p[1] - 2.0).abs() < 1e-10); // 50% of 4.0
    }

    #[test]
    fn skeletal() {
        let mut mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        // Two bones: identity and translate-X
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "BoneWeights",
                vec![1.0, 0.0, 0.0, 1.0],
                2,
            )));
        let bones = [identity_matrix(), translation_matrix(5.0, 0.0, 0.0)];
        let result = skeletal_transform(&mesh, &bones, "BoneWeights");
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10); // fully bone 0 (identity)
        let p1 = result.points.get(1);
        assert!((p1[0] - 5.0).abs() < 1e-10); // fully bone 1 (translate)
    }

    #[test]
    fn empty() {
        let result = weighted_transform(&PolyData::new(), &[]);
        assert_eq!(result.points.len(), 0);
    }
}
