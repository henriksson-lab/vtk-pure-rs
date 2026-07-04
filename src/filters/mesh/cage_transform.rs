//! Cage-based mesh deformation using inverse-distance cage weights.

use crate::data::{Points, PolyData};

/// Deform a mesh using a cage (control mesh) with inverse-distance weights.
///
/// `cage_original` and `cage_deformed` define the cage before and after deformation.
/// Each interior vertex is displaced by the weighted average of cage vertex displacements.
pub fn cage_deform(
    mesh: &PolyData,
    cage_original: &[[f64; 3]],
    cage_deformed: &[[f64; 3]],
) -> PolyData {
    let n = mesh.points.len();
    let nc = cage_original.len().min(cage_deformed.len());
    if n == 0 || nc == 0 {
        return mesh.clone();
    }

    let displacements: Vec<[f64; 3]> = (0..nc)
        .map(|i| {
            [
                cage_deformed[i][0] - cage_original[i][0],
                cage_deformed[i][1] - cage_original[i][1],
                cage_deformed[i][2] - cage_original[i][2],
            ]
        })
        .collect();

    let mut new_pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        // Compute inverse-distance weights to cage vertices
        let mut weights = Vec::with_capacity(nc);
        let mut w_sum = 0.0;
        for j in 0..nc {
            let d = ((p[0] - cage_original[j][0]).powi(2)
                + (p[1] - cage_original[j][1]).powi(2)
                + (p[2] - cage_original[j][2]).powi(2))
            .sqrt();
            let w = if d < 1e-15 { 1e10 } else { 1.0 / (d * d) };
            weights.push(w);
            w_sum += w;
        }
        let mut dp = [0.0; 3];
        if w_sum > 1e-15 {
            for j in 0..nc {
                let nw = weights[j] / w_sum;
                for c in 0..3 {
                    dp[c] += nw * displacements[j][c];
                }
            }
        }
        new_pts.push([p[0] + dp[0], p[1] + dp[1], p[2] + dp[2]]);
    }
    let mut result = mesh.clone();
    result.points = new_pts;
    result
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
    fn identity_cage() {
        let mesh = PolyData::from_points(vec![[0.5, 0.5, 0.5]]);
        let cage = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let result = cage_deform(&mesh, &cage, &cage); // no displacement
        let p = result.points.get(0);
        assert!((p[0] - 0.5).abs() < 0.01);
    }
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
