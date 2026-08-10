//! Cage-based mesh deformation using inverse-distance cage weights.

use crate::data::PolyData;

/// Deform a mesh using a cage (control mesh) with inverse-distance weights.
///
/// `cage_original` and `cage_deformed` define the cage before and after deformation.
/// Each interior vertex is displaced by the weighted average of cage vertex displacements.
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::mesh::cage_deform::cage_deform`], with the inverse-distance
/// exponent fixed at 2.
pub fn cage_deform(
    mesh: &PolyData,
    cage_original: &[[f64; 3]],
    cage_deformed: &[[f64; 3]],
) -> PolyData {
    crate::filters::mesh::cage_deform::cage_deform(mesh, cage_original, cage_deformed, 2.0)
}

/// Deform mesh using a bounding-box cage (8 control points).
pub fn bbox_cage_deform(mesh: &PolyData, deformed_corners: &[[f64; 3]; 8]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
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

    let original = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    cage_deform(mesh, &original, deformed_corners)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn translate_cage() {
        let mesh = PolyData::from_points(vec![[0.5, 0.5, 0.0]]);
        let orig = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let moved = [
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]; // all shifted +1 in X
        let result = cage_deform(&mesh, &orig, &moved);
        let p = result.points.get(0);
        assert!((p[0] - 1.5).abs() < 0.1); // should have moved ~+1 in X
    }
}
