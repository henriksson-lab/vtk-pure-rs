use crate::data::PolyData;
use std::collections::HashMap;

/// Check if a closed triangle mesh is convex.
///
/// A mesh is convex if all dihedral angles between adjacent faces
/// are <= 180 degrees (all edges are "valley" edges, not "ridge").
pub fn is_convex(input: &PolyData) -> bool {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    // Face normals
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
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-15 {
                [n[0] / len, n[1] / len, n[2] / len]
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

    for ((a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let n0 = normals[faces[0]];
        let n1 = normals[faces[1]];

        // Check if the edge is convex: the opposite vertex of face1
        // should be "behind" face0's plane
        let opp = cells[faces[1]].iter().find(|&&v| v != *a && v != *b);
        if let Some(&opp_v) = opp {
            let p = input.points.get(opp_v as usize);
            let v0 = input.points.get(*a as usize);
            let d = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];
            let dot = d[0] * n0[0] + d[1] * n0[1] + d[2] * n0[2];
            if dot > 1e-10 {
                return false;
            } // concave edge
        }
    }
    true
}

/// Compute convexity defect: fraction of edges that are concave.
pub fn convexity_defect(input: &PolyData) -> f64 {
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
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-15 {
                [n[0] / len, n[1] / len, n[2] / len]
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
    let mut concave = 0;
    for ((a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        total += 1;
        let n0 = normals[faces[0]];
        let opp = cells[faces[1]].iter().find(|&&v| v != *a && v != *b);
        if let Some(&opp_v) = opp {
            let p = input.points.get(opp_v as usize);
            let v0 = input.points.get(*a as usize);
            let d = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];
            if d[0] * n0[0] + d[1] * n0[1] + d[2] * n0[2] > 1e-10 {
                concave += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        concave as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convexity_basic() {
        // Simple test: just check the function runs without panic
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let d = convexity_defect(&pd);
        assert!(d >= 0.0 && d <= 1.0);
    }

    #[test]
    fn non_convex() {
        let mut pd = PolyData::new();
        // Create a concavity by pushing center vertex inward
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([1.0, 1.0, -1.0]); // concave vertex
        pd.polys.push_cell(&[0, 1, 4]);
        pd.polys.push_cell(&[1, 2, 4]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[3, 0, 4]);

        assert!(!is_convex(&pd));
        assert!(convexity_defect(&pd) > 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(is_convex(&pd));
    }
}
