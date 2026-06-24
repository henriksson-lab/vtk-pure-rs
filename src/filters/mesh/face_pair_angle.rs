use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Compute mean and max dihedral angle for the entire mesh.
pub fn dihedral_angle_stats(input: &PolyData) -> (f64, f64, f64) {
    // (min,max,mean)
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(c[0] as usize);
            let v1 = input.points.get(c[1] as usize);
            let v2 = input.points.get(c[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                [n[0] / l, n[1] / l, n[2] / l]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut angles = Vec::new();
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            let na = normals[faces[0]];
            let nb = normals[faces[1]];
            let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
            angles.push(dot.acos().to_degrees());
        }
    }

    if angles.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let min = angles.iter().copied().fold(f64::MAX, f64::min);
    let max = angles.iter().copied().fold(0.0f64, f64::max);
    let mean: f64 = angles.iter().sum::<f64>() / angles.len() as f64;
    (min, max, mean)
}

/// Compute the smoothness index: 1 - (max_dihedral / 180).
/// 1.0 = perfectly flat, 0.0 = has 180° fold.
pub fn smoothness_index(input: &PolyData) -> f64 {
    let (_, max, _) = dihedral_angle_stats(input);
    1.0 - (max / 180.0).min(1.0)
}

/// Compute the flatness score: fraction of edges with dihedral < threshold.
pub fn flatness_score(input: &PolyData, threshold_deg: f64) -> f64 {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(c[0] as usize);
            let v1 = input.points.get(c[1] as usize);
            let v2 = input.points.get(c[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                [n[0] / l, n[1] / l, n[2] / l]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut total = 0;
    let mut flat = 0;
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            total += 1;
            let na = normals[faces[0]];
            let nb = normals[faces[1]];
            let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
            if dot.acos().to_degrees() < threshold_deg {
                flat += 1;
            }
        }
    }

    if total > 0 {
        flat as f64 / total as f64
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_surface() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (min, max, mean) = dihedral_angle_stats(&pd);
        assert!(max < 5.0); // nearly flat
        assert!(smoothness_index(&pd) > 0.9);
        assert_eq!(flatness_score(&pd, 10.0), 1.0);
    }

    #[test]
    fn sharp_edge() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let (_, max, _) = dihedral_angle_stats(&pd);
        assert!(max > 30.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (min, max, mean) = dihedral_angle_stats(&pd);
        assert_eq!(min + max + mean, 0.0);
    }
}
